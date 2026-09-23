/// Group `text` into sentence-bounded chunks whose UTF-8 character length is
/// at most `max_chars`. Each chunk is a `String` ready to send to VoiceStudio.
///
/// Sentences are obtained from sentencex using `language` (ISO 639-1).
/// If a single sentence exceeds `max_chars`, it is split on whitespace into
/// word-bounded pieces. If even a single word exceeds `max_chars`, it is
/// hard-split at character boundaries so we never produce an oversized
/// request — VoiceStudio rejects anything over 4096 chars.
pub fn chunk_text(text: &str, language: &str, max_chars: usize) -> Vec<String> {
    assert!(max_chars > 0, "max_chars must be > 0");

    let sentences: Vec<&str> = sentencex::segment(language, text);
    let mut chunks: Vec<String> = Vec::new();
    let mut buf = String::new();

    for sentence in sentences {
        let s = sentence.trim();
        if s.is_empty() {
            continue;
        }

        let s_chars = s.chars().count();
        if s_chars > max_chars {
            // Flush whatever we had so the oversized sentence starts fresh.
            if !buf.is_empty() {
                chunks.push(std::mem::take(&mut buf));
            }
            chunks.extend(hard_split(s, max_chars));
            continue;
        }

        if buf.is_empty() {
            buf.push_str(s);
        } else if buf.chars().count() + 1 + s_chars <= max_chars {
            buf.push(' ');
            buf.push_str(s);
        } else {
            chunks.push(std::mem::take(&mut buf));
            buf.push_str(s);
        }
    }

    if !buf.is_empty() {
        chunks.push(buf);
    }
    chunks
}

/// Split `s` on whitespace into pieces of at most `max_chars` chars.
/// If a single "word" still exceeds `max_chars`, hard-split it.
fn hard_split(s: &str, max_chars: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();

    for word in s.split_whitespace() {
        let w_chars = word.chars().count();
        if w_chars > max_chars {
            if !buf.is_empty() {
                out.push(std::mem::take(&mut buf));
            }
            out.extend(force_split_chars(word, max_chars));
            continue;
        }

        if buf.is_empty() {
            buf.push_str(word);
        } else if buf.chars().count() + 1 + w_chars <= max_chars {
            buf.push(' ');
            buf.push_str(word);
        } else {
            out.push(std::mem::take(&mut buf));
            buf.push_str(word);
        }
    }

    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

/// Hard-split a single very long "word" at character boundaries.
fn force_split_chars(s: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for c in s.chars() {
        buf.push(c);
        if buf.chars().count() == max_chars {
            out.push(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sentence_stays_single() {
        let chunks = chunk_text("Hello world.", "en", 100);
        assert_eq!(chunks, vec!["Hello world."]);
    }

    #[test]
    fn groups_sentences_under_limit() {
        let text = "First sentence. Second sentence. Third sentence.";
        let chunks = chunk_text(text, "en", 100);
        assert_eq!(chunks, vec!["First sentence. Second sentence. Third sentence."]);
    }

    #[test]
    fn splits_when_exceeding_limit() {
        // "First sentence." (14) + " " + "Second sentence." (15) = 30 > 25,
        // so each sentence must end up in its own chunk.
        let text = "First sentence. Second sentence. Third sentence.";
        let chunks = chunk_text(text, "en", 25);
        assert_eq!(chunks.len(), 3);
        for c in &chunks {
            assert!(c.chars().count() <= 25, "chunk too big: {c:?}");
        }
        assert_eq!(chunks[0], "First sentence.");
        assert_eq!(chunks[1], "Second sentence.");
        assert_eq!(chunks[2], "Third sentence.");
    }

    #[test]
    fn handles_oversized_single_sentence() {
        // 80-char sentence, max 30 → must split on word boundaries.
        let long = "This is a deliberately very long sentence that exceeds the maximum chunk size easily.";
        let chunks = chunk_text(long, "en", 30);
        assert!(chunks.len() > 1);
        for c in &chunks {
            assert!(c.chars().count() <= 30, "chunk too big: {c:?}");
        }
        // Rejoining the chunks recovers the original text.
        let joined: String = chunks.join(" ");
        assert_eq!(joined.split_whitespace().collect::<Vec<_>>(), long.split_whitespace().collect::<Vec<_>>());
    }

    #[test]
    fn joins_sentences_with_single_space() {
        // sentencex keeps the standalone period as its own segment;
        // the chunker must rejoin non-empty segments with a single space.
        let chunks = chunk_text("Hello world.   . Goodbye.", "en", 100);
        assert_eq!(chunks, vec!["Hello world. . Goodbye."]);
    }

    #[test]
    fn skips_empty_segments() {
        // Whitespace-only input: sentencex may yield an empty segment,
        // which the chunker must drop instead of producing a 0-char request.
        let chunks = chunk_text("   ", "en", 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn empty_input_yields_no_chunks() {
        let chunks = chunk_text("", "en", 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn force_split_handles_oversized_word() {
        // A single 20-char "word" with max_chars 5 must be hard-split.
        let pieces = force_split_chars("abcdefghijklmnopqrst", 5);
        assert_eq!(pieces, vec!["abcde", "fghij", "klmno", "pqrst"]);
    }
}
