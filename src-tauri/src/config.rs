use serde::{Deserialize, Serialize};
use std::path::Path;

/// Config carregada de %LOCALAPPDATA%\hawkeye-pi\config.local.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub cam_host: String,
    #[serde(default = "default_port")]
    pub cam_port: u16,
    #[serde(default = "default_user")]
    pub cam_user: String,
    #[serde(default)]
    pub cam_pass: String,
    #[serde(default = "default_profile")]
    pub cam_profile: String,
    #[serde(default = "default_label")]
    pub cam_label: String,
    #[serde(default = "default_stream")]
    pub stream_name: String,
}

fn default_port() -> u16 { 5000 }
fn default_user() -> String { "admin".into() }
fn default_profile() -> String { "IPCProfilesToken0".into() }
fn default_label() -> String { "Camera".into() }
fn default_stream() -> String { "garagem_web".into() }

impl Config {
    pub fn load_or_default(dir: &Path) -> Self {
        let p = dir.join("config.local.yaml");
        if !p.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&p) {
            Ok(s) => parse_simple_yaml(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let p = dir.join("config.local.yaml");
        let s = format!(
            "cam_host: {}\ncam_port: {}\ncam_user: {}\ncam_pass: {}\ncam_profile: {}\ncam_label: {}\nstream_name: {}\n",
            self.cam_host, self.cam_port, self.cam_user, self.cam_pass,
            self.cam_profile, self.cam_label, self.stream_name
        );
        std::fs::write(p, s)
    }

    pub fn is_valid(&self) -> bool {
        !self.cam_host.is_empty() && !self.cam_pass.is_empty()
    }
}

/// Parser MUITO simples de YAML no formato "chave: valor" - sem deps externas.
fn parse_simple_yaml(s: &str) -> Option<Config> {
    let mut c = Config::default();
    for line in s.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() || !line.contains(':') { continue; }
        let mut it = line.splitn(2, ':');
        let k = it.next()?.trim();
        let v = it.next()?.trim().trim_matches('"').trim_matches('\'').to_string();
        match k {
            "cam_host" => c.cam_host = v,
            "cam_port" => c.cam_port = v.parse().unwrap_or(5000),
            "cam_user" => c.cam_user = v,
            "cam_pass" => c.cam_pass = v,
            "cam_profile" => c.cam_profile = v,
            "cam_label" => c.cam_label = v,
            "stream_name" => c.stream_name = v,
            _ => {}
        }
    }
    Some(c)
}
