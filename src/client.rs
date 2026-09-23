use anyhow::{anyhow, Result};
use serde::Serialize;
use std::time::Duration;

/// VoiceStudio's OpenAI-compatible text-to-speech endpoint.
const SPEECH_PATH: &str = "/v1/audio/speech";

/// TTS parameters shared by every request. Mirrors the fields VoiceStudio's
/// `/v1/audio/speech` JSON body expects.
#[derive(Debug, Clone)]
pub struct TtsOptions<'a> {
    pub model: &'a str,
    pub voice: &'a str,
    pub response_format: &'a str,
    pub language: &'a str,
    pub speed: f32,
}

/// JSON body sent to `/v1/audio/speech`.
#[derive(Debug, Serialize)]
struct SpeechRequest<'a> {
    model: &'a str,
    input: &'a str,
    voice: &'a str,
    response_format: &'a str,
    speed: f32,
    language: &'a str,
}

/// Send `text` to VoiceStudio and return the raw audio bytes for one chunk.
///
/// `base_url` is the server root (e.g. `http://localhost:3900`); the
/// `/v1/audio/speech` path is appended here. `api_key` is the bearer token
/// for non-loopback servers; loopback requests do not need one.
pub fn synthesize(
    base_url: &str,
    api_key: Option<&str>,
    opts: &TtsOptions<'_>,
    text: &str,
) -> Result<Vec<u8>> {
    let url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        SPEECH_PATH.trim_start_matches('/')
    );

    let body = SpeechRequest {
        model: opts.model,
        input: text,
        voice: opts.voice,
        response_format: opts.response_format,
        speed: opts.speed,
        language: opts.language,
    };

    let mut req = ureq::post(&url)
        .timeout(Duration::from_secs(600))
        .set("Content-Type", "application/json");
    if let Some(key) = api_key {
        if !key.is_empty() {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
    }

    let response = req.send_json(body).map_err(|e| {
        anyhow!(
            "request to {url} failed: {e} \
             (is VoiceStudio running and reachable at {base_url}?)"
        )
    })?;

    if response.status() != 200 {
        let status = response.status();
        let body = response
            .into_string()
            .unwrap_or_else(|_| "<unreadable>".to_string());
        return Err(anyhow!("VoiceStudio returned HTTP {status}: {body}"));
    }

    let mut out = Vec::new();
    let mut reader = response.into_reader();
    std::io::Read::read_to_end(&mut reader, &mut out)
        .map_err(|e| anyhow!("failed to read audio response: {e}"))?;
    Ok(out)
}
