mod chunk;
mod cli;
mod client;
mod slice;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::Args;
use crate::client::TtsOptions;

fn main() -> Result<()> {
    let args = Args::parse();

    // Default output: same directory as the input, same stem, format extension.
    let output: PathBuf = args.output.clone().unwrap_or_else(|| {
        let stem = args
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = args.input.parent().unwrap_or_else(|| {
            std::path::Path::new(".")
        });
        parent.join(format!("{stem}.{}", args.format.extension()))
    });

    let text = slice::read_line_range(&args.input, args.from, args.to)?;
    let chunks = chunk::chunk_text(&text, &args.language, args.max_chars);

    if chunks.is_empty() {
        anyhow::bail!(
            "no text to convert (selected range produced 0 sentences) \
             — check the line range and --language"
        );
    }

    let format_str = args.format.as_str();
    let opts = TtsOptions {
        model: &args.model,
        voice: &args.voice,
        response_format: format_str,
        language: &args.language,
        speed: args.speed,
    };

    let mut out = File::create(&output)
        .with_context(|| format!("failed to create {}", output.display()))?;
    let mut total_bytes: usize = 0;

    for (i, c) in chunks.iter().enumerate() {
        eprintln!(
            "synthesizing chunk {}/{} ({} chars)...",
            i + 1,
            chunks.len(),
            c.chars().count()
        );
        let bytes = client::synthesize(
            &args.base_url,
            args.api_key.as_deref(),
            &opts,
            c,
        )?;
        out.write_all(&bytes)
            .with_context(|| format!("failed to write to {}", output.display()))?;
        total_bytes += bytes.len();
    }

    println!(
        "wrote {} bytes from {} chunk(s) to {}",
        total_bytes,
        chunks.len(),
        output.display()
    );
    Ok(())
}
