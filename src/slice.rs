use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

/// Read `path` as text and return the lines from `from_line` to `to_line`
/// (both 1-indexed, inclusive), joined back together with `\n` so the
/// resulting string preserves the original line structure.
///
/// `to_line = None` means "to the end of the file".
///
/// Validates that:
/// - the file is non-empty,
/// - `from_line >= 1`,
/// - if `to_line` is given, `to_line >= from_line`,
/// - both indices fall within the actual line count.
pub fn read_line_range(path: &Path, from_line: usize, to_line: Option<usize>) -> Result<String> {
    if from_line == 0 {
        return Err(anyhow!("--from must be >= 1 (got 0)"));
    }
    if let Some(to) = to_line {
        if to < from_line {
            return Err(anyhow!("--to ({to}) must be >= --from ({from_line})"));
        }
    }

    let raw = fs::read_to_string(path)
        .map_err(|e| anyhow!("failed to read {}: {e}", path.display()))?;

    // `split('\n')` preserves empty trailing lines and the structure of the
    // file better than `lines()`, which drops a trailing empty line.
    let parts: Vec<&str> = raw.split('\n').collect();
    let total = parts.len();

    if total == 1 && parts[0].is_empty() {
        return Err(anyhow!("{} is empty", path.display()));
    }
    if from_line > total {
        return Err(anyhow!(
            "--from ({from_line}) is past the last line of the file ({total})"
        ));
    }
    if let Some(to) = to_line {
        if to > total {
            return Err(anyhow!(
                "--to ({to}) is past the last line of the file ({total})"
            ));
        }
    }

    // Convert 1-indexed inclusive bounds to a 0-indexed exclusive `slice` range.
    let start = from_line - 1;
    let end = to_line.unwrap_or(total); // exclusive upper bound
    Ok(parts[start..end].join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_tmp(content: &str) -> tempfile_lite::TempPath {
        tempfile_lite::create_with(content)
    }

    #[test]
    fn reads_full_range() {
        let f = write_tmp("a\nb\nc");
        let out = read_line_range(&f.path, 1, Some(3)).unwrap();
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn reads_partial_range() {
        let f = write_tmp("a\nb\nc\nd");
        let out = read_line_range(&f.path, 2, Some(3)).unwrap();
        assert_eq!(out, "b\nc");
    }

    #[test]
    fn none_to_means_end_of_file() {
        let f = write_tmp("a\nb\nc\nd");
        let out = read_line_range(&f.path, 2, None).unwrap();
        assert_eq!(out, "b\nc\nd");
    }

    #[test]
    fn preserves_trailing_empty_line() {
        let f = write_tmp("a\nb\n");
        // split('\n') yields ["a", "b", ""] - 3 lines.
        let out = read_line_range(&f.path, 1, Some(3)).unwrap();
        assert_eq!(out, "a\nb\n");
    }

    #[test]
    fn single_line_file() {
        let f = write_tmp("only one line");
        let out = read_line_range(&f.path, 1, Some(1)).unwrap();
        assert_eq!(out, "only one line");
    }

    #[test]
    fn rejects_zero_from() {
        let f = write_tmp("a\nb");
        assert!(read_line_range(&f.path, 0, Some(1)).is_err());
    }

    #[test]
    fn rejects_to_less_than_from() {
        let f = write_tmp("a\nb\nc");
        assert!(read_line_range(&f.path, 2, Some(1)).is_err());
    }

    #[test]
    fn rejects_from_past_end() {
        let f = write_tmp("a\nb");
        assert!(read_line_range(&f.path, 99, Some(100)).is_err());
    }

    #[test]
    fn rejects_to_past_end() {
        let f = write_tmp("a\nb");
        assert!(read_line_range(&f.path, 1, Some(99)).is_err());
    }

    #[test]
    fn rejects_empty_file() {
        let f = write_tmp("");
        assert!(read_line_range(&f.path, 1, Some(1)).is_err());
    }

    /// Minimal tempfile shim so the tests don't pull in the `tempfile` crate.
    mod tempfile_lite {
        use std::env;
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        pub struct TempPath {
            pub path: PathBuf,
            _file: File,
        }

        impl Drop for TempPath {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.path);
            }
        }

        pub fn create_with(content: &str) -> TempPath {
            let pid = std::process::id();
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = env::temp_dir().join(format!("tts-cli-test-{pid}-{nanos}.txt"));
            let mut file = File::create(&path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
            TempPath { path, _file: file }
        }
    }
}
