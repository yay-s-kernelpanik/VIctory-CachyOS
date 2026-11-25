//! RPF9 archive detection and (stub) extraction.
//!
//! This module does **not** implement RPF9 decompression. It detects the
//! archive by its magic bytes and, when an external extractor
//! (`rage-tools-vi`) is available on `PATH`, invokes it into a temporary
//! directory. Otherwise it reports that extraction is unavailable.

use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Errors produced while trying to extract an RPF9 archive.
#[derive(Debug)]
pub enum RpfError {
    /// No external extractor was found on `PATH`.
    ToolNotAvailable,
    /// The input or extracted data is malformed.
    Malformed(String),
    /// Underlying I/O failure.
    Io(std::io::Error),
}

impl std::fmt::Display for RpfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpfError::ToolNotAvailable => write!(f, "RPF9 extraction tool not available"),
            RpfError::Malformed(s) => write!(f, "malformed RPF9: {s}"),
            RpfError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for RpfError {}

/// True if `path` begins with an RPF9 magic (`RPF9` or `9FPR`).
pub fn is_rpf9(path: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 16];
    let n = file.read(&mut buf).unwrap_or(0);
    if n < 4 {
        return false;
    }
    if buf[..4] == *b"RPF9" || buf[..4] == *b"9FPR" {
        return true;
    }
    // Log the first 16 bytes in hex for diagnostics.
    let hex: String = buf[..n.min(16)]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("rpf9: not an RPF9 archive; first bytes: {hex}");
    false
}

/// Invoke the external extractor into a fresh temporary directory.
pub fn extract_to_temp(path: &Path) -> Result<tempfile::TempDir, RpfError> {
    if Command::new("rage-tools-vi")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        return Err(RpfError::ToolNotAvailable);
    }
    let temp = tempfile::tempdir().map_err(RpfError::Io)?;
    let output = Command::new("rage-tools-vi")
        .arg("extract")
        .arg(path)
        .arg(temp.path())
        .output()
        .map_err(RpfError::Io)?;
    if !output.status.success() {
        return Err(RpfError::Malformed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(temp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_rpf9_with_fake_magic() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.rpf");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"RPF9").unwrap();
        f.write_all(b"junk junk junk").unwrap();
        assert!(is_rpf9(&p));
    }

    #[test]
    fn rejects_non_rpf9() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("b.rpf");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"XXXX").unwrap();
        f.write_all(b"junk").unwrap();
        assert!(!is_rpf9(&p));
    }

    #[test]
    fn tool_not_available_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("c.rpf");
        std::fs::write(&p, b"RPF9").unwrap();
        let err = extract_to_temp(&p).unwrap_err();
        assert!(matches!(err, RpfError::ToolNotAvailable));
    }
}