# tts-cli

A Rust CLI that converts text files to audio using [VoiceStudio](https://voicestudio.sh)'s local API. Optionally limit the conversion to a specific range of lines.

## Features

- Reads a text file and POSTs it to VoiceStudio's OpenAI-compatible `/v1/audio/speech` endpoint.
- Select a subset of the file by `--from` / `--to` (1-indexed, inclusive).
- Long inputs are split with [sentencex](https://docs.rs/sentencex) into sentence-bounded chunks that fit VoiceStudio's per-request character cap.
- Configurable voice, model, language, speed, output format, server URL, and bearer token.

## Build

```bash
cargo build --release
```

The binary lands at `target/release/tts-cli`.

## Usage

```
tts-cli <INPUT> [OPTIONS]
```

| Flag | Description |
|---|---|
| `<INPUT>` | Path to the text file to convert (required). |
| `-o, --output <FILE>` | Output audio path. Default: `<input>.<format>` in the same directory. |
| `--from <LINE>` | First line to include, 1-indexed, inclusive. Default: `1`. |
| `--to <LINE>` | Last line to include, 1-indexed, inclusive. Default: last line. |
| `--base-url <URL>` | VoiceStudio base URL. Default: `http://localhost:3900`. |
| `--voice <VOICE>` | Voice id or preset (e.g. `default`, `alloy`, `echo`). Default: `default`. |
| `--format <FMT>` | `mp3` \| `opus` \| `aac` \| `flac` \| `wav` \| `pcm`. Default: `mp3`. |
| `--model <MODEL>` | Engine id. Default: `omnivoice`. |
| `--language <CODE>` | ISO 639-1 code passed to both sentencex and VoiceStudio. Default: `en`. |
| `--speed <FLOAT>` | Playback speed (0.25–4.0). Default: `1.0`. |
| `--api-key <KEY>` | Bearer token for non-loopback servers. Loopback requests need no token. |
| `--max-chars <N>` | Per-request character budget (must stay under 4096). Default: `4000`. |

Run `tts-cli --help` for the full reference.

## Examples

Convert a whole file to MP3 (output lands next to the input as `book.mp3`):

```bash
tts-cli book.txt
```

Convert only lines 5–20:

```bash
tts-cli book.txt --from 5 --to 20 -o chapter_one.mp3
```

Use a different voice and slower speed:

```bash
tts-cli book.txt --voice alloy --speed 0.9 --format wav
```

Talk to a remote VoiceStudio server:

```bash
tts-cli book.txt \
  --base-url https://voicestudio.example.com \
  --api-key "$OMNIVOICE_API_KEY"
```

## How it works

1. **Slice**: read the file and extract the line range the user asked for.
2. **Chunk**: run `sentencex::segment(language, text)` to split into sentences, then greedily group them into chunks under `--max-chars` (with a whitespace and character-level fallback for sentences that don't fit).
3. **Synthesize**: POST each chunk to `{base_url}/v1/audio/speech` with `Content-Type: application/json` and an optional `Authorization: Bearer <key>` header.
4. **Write**: append each response's raw bytes to the output file in order.

## Caveats

- **Concatenation of compressed formats**: when the input needs more than one chunk, the raw MP3/AAC/OPUS bytes are written back-to-back. Most players tolerate this, but it is not a strictly valid single stream. If you need a strictly-valid file, re-mux with `ffmpeg -i out.mp3 -c copy out_fixed.mp3`. WAV and PCM concatenate cleanly.
- **Per-request cap**: VoiceStudio rejects requests over 4096 characters. The default `--max-chars 4000` leaves headroom; do not raise it above 4096.
- **VoiceStudio must be running**: the CLI does not start or manage the desktop app.

## Tests

```bash
cargo test          # unit tests for line-range slicing and sentence chunking
cargo clippy --all-targets -- -D warnings
```

## Dependencies

- [`clap`](https://docs.rs/clap) — CLI argument parsing
- [`ureq`](https://docs.rs/ureq) — synchronous HTTP client
- [`sentencex`](https://docs.rs/sentencex) — multi-language sentence segmentation
- [`serde`](https://docs.rs/serde) / [`serde_json`](https://docs.rs/serde_json) — JSON body
- [`anyhow`](https://docs.rs/anyhow) — error handling
