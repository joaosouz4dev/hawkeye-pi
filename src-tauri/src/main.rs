#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod onvif;
mod server;
mod go2rtc;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tokio::sync::RwLock;

pub struct AppState {
    pub cfg: RwLock<config::Config>,
    pub go2rtc_url: String,
    pub api_url: String,
}

fn main() {
    // Diretório de dados do app (%LOCALAPPDATA%\hawkeye-pi\ no Windows)
    let data_dir = dirs::data_local_dir()
        .expect("sem LOCALAPPDATA")
        .join("hawkeye-pi");
    std::fs::create_dir_all(&data_dir).expect("nao criou pasta de dados");

    // Carrega config existente ou cria vazia (first-run)
    let cfg = config::Config::load_or_default(&data_dir);
    let first_run = !cfg.is_valid();

    // Portas locais aleatorias (loopback, invisiveis pra fora)
    let go2rtc_port = server::random_free_port().unwrap_or(1984);
    let api_port = server::random_free_port().unwrap_or(1985);

    let go2rtc_url = format!("http://127.0.0.1:{}", go2rtc_port);
    let api_url = format!("http://127.0.0.1:{}", api_port);

    // Grava go2rtc.yaml em data_dir com a porta escolhida e credenciais atuais
    let yaml_path = data_dir.join("go2rtc.yaml");
    go2rtc::write_config(&yaml_path, &cfg, go2rtc_port).ok();

    let state = Arc::new(AppState {
        cfg: RwLock::new(cfg),
        go2rtc_url: go2rtc_url.clone(),
        api_url: api_url.clone(),
    });

    // Roda o servidor HTTP local (axum) numa thread tokio dedicada
    let state_srv = state.clone();
    let data_dir_srv = data_dir.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("nao criou runtime");
        rt.block_on(async move {
            server::run(api_port, state_srv, data_dir_srv).await;
        });
    });

    let state_tauri = state.clone();
    let data_dir_tauri = data_dir.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state_tauri.clone())
        .setup(move |app| {
            // Spawna o go2rtc.exe (sidecar) com a config gerada
            let shell = app.shell();
            let yaml = data_dir_tauri.join("go2rtc.yaml");
            let (mut rx, child) = shell
                .sidecar("go2rtc")
                .expect("sidecar go2rtc nao encontrado")
                .args(["-c", yaml.to_str().unwrap()])
                .spawn()
                .expect("nao rodou go2rtc");
            // guarda o handle para matar depois
            app.manage(go2rtc::ChildHandle(std::sync::Mutex::new(Some(child))));

            // consome stdout/stderr do go2rtc pra nao bloquear
            tauri::async_runtime::spawn(async move {
                while let Some(_event) = rx.recv().await {
                    // silencioso; se quiser logar em file, faz aqui
                }
            });

            // Aponta a janela para o servidor HTTP local (nao usa asset://).
            // Espera o axum aceitar conexao antes de navegar, senao da "not found"/erro.
            let win = app.get_webview_window("main").expect("janela main");
            let url = format!("{}/", state_tauri.api_url);
            let addr = state_tauri
                .api_url
                .trim_start_matches("http://")
                .to_string();
            tauri::async_runtime::spawn(async move {
                for _ in 0..100 {
                    if std::net::TcpStream::connect(&addr).is_ok() {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                let _ = win.eval(&format!("window.location.replace('{}');", url));
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // mata go2rtc ao fechar
                if let Some(h) = window.app_handle().try_state::<go2rtc::ChildHandle>() {
                    if let Ok(mut opt) = h.0.lock() {
                        if let Some(child) = opt.take() {
                            let _ = child.kill();
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("erro ao rodar app tauri");

    // silencia warnings
    let _ = first_run;
}
