use axum::{
    body::Body,
    extract::{Path as UrlPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::TcpListener, path::PathBuf, sync::Arc};

use crate::{onvif, AppState};

pub fn random_free_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|l| l.local_addr().ok())
        .map(|a| a.port())
}

pub async fn run(port: u16, state: Arc<AppState>, data_dir: PathBuf) {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/config", get(get_config))
        .route("/api/setup", post(post_setup))
        .route("/move/:dir", post(move_ptz))
        .route("/stop", post(stop_ptz))
        .route("/*path", get(serve_static))
        .with_state(ServerState {
            app: state,
            data_dir,
        });

    let addr = format!("127.0.0.1:{}", port);
    if let Ok(listener) = tokio::net::TcpListener::bind(&addr).await {
        let _ = axum::serve(listener, app).await;
    }
}

#[derive(Clone)]
struct ServerState {
    app: Arc<AppState>,
    data_dir: PathBuf,
}

#[derive(Serialize)]
struct ConfigResponse {
    go2rtc: String,
    stream: String,
    label: String,
    first_run: bool,
}

async fn get_config(State(s): State<ServerState>) -> Json<ConfigResponse> {
    let cfg = s.app.cfg.read().await;
    Json(ConfigResponse {
        go2rtc: s.app.go2rtc_url.clone(),
        stream: cfg.stream_name.clone(),
        label: cfg.cam_label.clone(),
        first_run: !cfg.is_valid(),
    })
}

#[derive(Deserialize)]
struct SetupBody {
    cam_host: String,
    cam_user: String,
    cam_pass: String,
    cam_label: String,
    #[serde(default)]
    cam_profile: Option<String>,
}

async fn post_setup(
    State(s): State<ServerState>,
    Json(body): Json<SetupBody>,
) -> Json<serde_json::Value> {
    let mut cfg = s.app.cfg.write().await;
    cfg.cam_host = body.cam_host;
    cfg.cam_user = body.cam_user;
    cfg.cam_pass = body.cam_pass;
    cfg.cam_label = body.cam_label;
    if let Some(p) = body.cam_profile { cfg.cam_profile = p; }
    let ok = cfg.save(&s.data_dir).is_ok();
    Json(serde_json::json!({ "ok": ok }))
}

async fn move_ptz(
    State(s): State<ServerState>,
    UrlPath(dir): UrlPath<String>,
) -> Json<serde_json::Value> {
    let (x, y) = match dir.as_str() {
        "up"        => (0.0, 0.6),
        "down"      => (0.0, -0.6),
        "left"      => (-0.6, 0.0),
        "right"     => (0.6, 0.0),
        "upleft"    => (-0.6, 0.6),
        "upright"   => (0.6, 0.6),
        "downleft"  => (-0.6, -0.6),
        "downright" => (0.6, -0.6),
        _ => return Json(serde_json::json!({ "ok": false, "err": "direcao invalida" })),
    };
    let cfg = s.app.cfg.read().await.clone();
    let ok = onvif::ptz_move(&cfg, x, y).await;
    Json(serde_json::json!({ "ok": ok }))
}

async fn stop_ptz(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let cfg = s.app.cfg.read().await.clone();
    let ok = onvif::ptz_stop(&cfg).await;
    Json(serde_json::json!({ "ok": ok }))
}

// ------------- servir arquivos estaticos EMBUTIDOS no binario -------------
//
// Usamos include_bytes! em tempo de compilacao. O .exe fica auto-contido -
// nao depende de nenhum arquivo no filesystem ao rodar.

macro_rules! ui_asset {
    ($name:expr) => { include_bytes!(concat!("../../ui/", $name)) };
}

fn get_asset(rel: &str) -> Option<&'static [u8]> {
    match rel {
        "index.html"             => Some(ui_asset!("index.html")),
        "video-rtc.js"           => Some(ui_asset!("video-rtc.js")),
        "video-stream-muted.js"  => Some(ui_asset!("video-stream-muted.js")),
        "manifest.webmanifest"   => Some(ui_asset!("manifest.webmanifest")),
        "sw.js"                  => Some(ui_asset!("sw.js")),
        "favicon.png"            => Some(ui_asset!("favicon.png")),
        "favicon.ico"            => Some(ui_asset!("favicon.png")),
        "icon-192.png"           => Some(ui_asset!("icon-192.png")),
        "icon-512.png"           => Some(ui_asset!("icon-512.png")),
        "icon-mask.png"          => Some(ui_asset!("icon-mask.png")),
        _ => None,
    }
}

async fn serve_index() -> Response {
    serve_asset("index.html")
}

async fn serve_static(UrlPath(path): UrlPath<String>) -> Response {
    serve_asset(&path)
}

fn serve_asset(rel: &str) -> Response {
    if let Some(bytes) = get_asset(rel) {
        let ct = guess_mime(rel);
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, ct)
            .body(Body::from(&bytes[..]))
            .unwrap()
            .into_response();
    }
    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn guess_mime(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".html") { "text/html; charset=utf-8" }
    else if lower.ends_with(".js") { "text/javascript; charset=utf-8" }
    else if lower.ends_with(".css") { "text/css; charset=utf-8" }
    else if lower.ends_with(".png") { "image/png" }
    else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "image/jpeg" }
    else if lower.ends_with(".webmanifest") || lower.ends_with(".json") { "application/manifest+json; charset=utf-8" }
    else if lower.ends_with(".ico") { "image/x-icon" }
    else { "application/octet-stream" }
}
