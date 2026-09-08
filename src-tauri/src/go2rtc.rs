use crate::config::Config;
use std::path::Path;
use tauri_plugin_shell::process::CommandChild;

/// Handle do processo go2rtc pra matar ao fechar o app
pub struct ChildHandle(pub std::sync::Mutex<Option<CommandChild>>);

/// Escreve go2rtc.yaml em disco com base na config atual + porta escolhida
pub fn write_config(path: &Path, cfg: &Config, api_port: u16) -> std::io::Result<()> {
    // Se ainda nao tem credenciais (first-run), escreve config minima para o go2rtc
    // subir sem crashar - streams ficam vazios.
    let streams = if cfg.is_valid() {
        // Transcode por SOFTWARE (sem #hardware): a saida do encoder NVENC
        // sai com formato de pixel que o WebView2 renderiza como tela verde.
        format!(
            r#"streams:
  main:
    - "rtsp://{}:{}@{}:554/onvif1#rtptransport=udp"
    - "ffmpeg:main#video=h264"
  {}:
    - "ffmpeg:main#video=h264"
"#,
            cfg.cam_user, cfg.cam_pass, cfg.cam_host, cfg.stream_name
        )
    } else {
        // sem config, deixa streams vazio - a UI mostra tela de setup
        "streams: {}\n".to_string()
    };
    let yaml = format!(
        r#"{}
api:
  listen: ":{}"
  origin: "*"

rtsp:
  listen: ":8554"

webrtc:
  candidates:
    - stun:8555
"#,
        streams, api_port
    );
    std::fs::write(path, yaml)
}
