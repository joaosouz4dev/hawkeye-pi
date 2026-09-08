use base64::Engine;
use rand::RngCore;
use sha1::{Digest, Sha1};

use crate::config::Config;

/// Retorna true se a camera aceitou (200 OK) ou respondeu vazio (comportamento do Stop).
pub async fn ptz_move(cfg: &Config, x: f32, y: f32) -> bool {
    let ns = "http://www.onvif.org/ver20/ptz/wsdl";
    let sch = "http://www.onvif.org/ver10/schema";
    let inner = format!(
        r#"<ContinuousMove xmlns="{ns}"><ProfileToken>{tok}</ProfileToken><Velocity><PanTilt x="{x}" y="{y}" xmlns="{sch}"/></Velocity></ContinuousMove>"#,
        tok = cfg.cam_profile
    );
    soap_call(cfg, &format!("{}/ContinuousMove", ns), &inner).await
}

pub async fn ptz_stop(cfg: &Config) -> bool {
    let ns = "http://www.onvif.org/ver20/ptz/wsdl";
    let inner = format!(
        r#"<Stop xmlns="{ns}"><ProfileToken>{tok}</ProfileToken><PanTilt>true</PanTilt><Zoom>true</Zoom></Stop>"#,
        tok = cfg.cam_profile
    );
    soap_call(cfg, &format!("{}/Stop", ns), &inner).await
}

async fn soap_call(cfg: &Config, action: &str, inner: &str) -> bool {
    let hdr = wss_header(&cfg.cam_user, &cfg.cam_pass);
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope">{hdr}<s:Body>{inner}</s:Body></s:Envelope>"#
    );
    let url = format!("http://{}:{}/onvif/ptz_service", cfg.cam_host, cfg.cam_port);
    let ct = format!(r#"application/soap+xml; charset=utf-8; action="{action}""#);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    match client.post(&url).header("Content-Type", ct).body(body).send().await {
        Ok(r) => {
            // camera as vezes fecha sem corpo (Stop) - tratamos como sucesso se a conexao completou
            r.status().is_success() || r.status().as_u16() == 0
        }
        Err(_) => false,
    }
}

fn wss_header(user: &str, pass: &str) -> String {
    let mut nonce = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce);
    let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // digest = base64(sha1(nonce + created + password))
    let mut h = Sha1::new();
    h.update(nonce);
    h.update(created.as_bytes());
    h.update(pass.as_bytes());
    let digest = base64::engine::general_purpose::STANDARD.encode(h.finalize());
    let n64 = base64::engine::general_purpose::STANDARD.encode(nonce);

    format!(
        r#"<s:Header><Security s:mustUnderstand="1" xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><UsernameToken><Username>{user}</Username><Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">{digest}</Password><Nonce EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">{n64}</Nonce><Created xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">{created}</Created></UsernameToken></Security></s:Header>"#
    )
}
