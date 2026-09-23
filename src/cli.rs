use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Convert a text file to audio using VoiceStudio's local API.
#[derive(Debug, Parser)]
#[command(name = "tts-cli", version, about, long_about = None)]
pub struct Args {
    /// Input text file to convert.
    pub input: PathBuf,

    /// Output audio file (default: <input>.<format> in the same directory).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// First line to include (1-indexed, inclusive).
    #[arg(long, default_value_t = 1)]
    pub from: usize,

    /// Last line to include (1-indexed, inclusive).
    #[arg(long)]
    pub to: Option<usize>,

    /// VoiceStudio base URL.
    #[arg(long, default_value = "http://localhost:3900")]
    pub base_url: String,

    /// Voice id or preset (e.g. "default", "alloy", "echo").
    #[arg(long, default_value = "default")]
    pub voice: String,

    /// Audio output format.
    #[arg(long, value_enum, default_value_t = AudioFormat::Mp3)]
    pub format: AudioFormat,

    /// Engine / model id.
    #[arg(long, default_value = "omnivoice")]
    pub model: String,

    /// ISO 639-1 language code (passed to both sentencex and VoiceStudio).
    #[arg(long, default_value = "en")]
    pub language: String,

    /// Playback speed multiplier (0.25 - 4.0).
    #[arg(long, default_value_t = 1.0)]
    pub speed: f32,

    /// Bearer token for non-loopback VoiceStudio servers.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Per-request character budget. Must stay under VoiceStudio's 4096 cap.
    #[arg(long, default_value_t = 4000)]
    pub max_chars: usize,
}

/// Audio output formats supported by VoiceStudio's /v1/audio/speech endpoint.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AudioFormat {
    Mp3,
    Opus,
    Aac,
    Flac,
    Wav,
    Pcm,
}

impl AudioFormat {
    /// Wire value sent in the JSON body's `response_format` field.
    pub fn as_str(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Opus => "opus",
            AudioFormat::Aac => "aac",
            AudioFormat::Flac => "flac",
            AudioFormat::Wav => "wav",
            AudioFormat::Pcm => "pcm",
        }
    }

    /// Conventional file extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Opus => "opus",
            AudioFormat::Aac => "aac",
            AudioFormat::Flac => "flac",
            AudioFormat::Wav => "wav",
            AudioFormat::Pcm => "pcm",
        }
    }
}
