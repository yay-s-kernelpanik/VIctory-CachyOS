//! Glue for RPF9 archives on Linux.
//!
//! This module does **not** implement RPF9 decompression. It detects the
//! archive by its magic bytes and, when an external extractor
//! (`rage-tools-vi`) is available on `PATH`, invokes it into a temporary
//! directory. Otherwise it reports that extraction is unavailable and leaves
//! the archive untouched, so `translate` can still emit its existing
//! "extract with rage-tools-vi first" note.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Errors produced while trying to extract an RPF9 archive.
#[derive(Debug)]
pub enum RpfError {
    /// No external extractor was found on `PATH`.
    ToolNotAvailable,
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// The extractor ran but failed.
    ExtractFailed(String),
}

impl std::fmt::Display for RpfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpfError::ToolNotAvailable => write!(f, "RPF9 extraction tool not available"),
            RpfError::Io(e) => write!(f, "I/O error: {e}"),
            RpfError::ExtractFailed(s) => write!(f, "extraction failed: {s}"),
        }
    }
}

impl std::error::Error for RpfError {}

/// True if `path` begins with an RPF9 magic (`RPF9` or `9FPR`).
pub fn is_rpf9(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 4];
    if file.read_exact(&mut buf).is_err() {
        return false;
    }
    buf == *b"RPF9" || buf == *b"9FPR"
}

/// True if an external extractor is available on `PATH`.
pub fn extractor_available() -> bool {
    Command::new("rage-tools-vi")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Invoke the external extractor into a fresh temporary directory.
pub fn extract_to_temp(rpf_path: &Path) -> Result<tempfile::TempDir, RpfError> {
    if !extractor_available() {
        return Err(RpfError::ToolNotAvailable);
    }
    let temp = tempfile::tempdir().map_err(RpfError::Io)?;
    let output = Command::new("rage-tools-vi")
        .arg("extract")
        .arg(rpf_path)
        .arg(temp.path())
        .output()
        .map_err(RpfError::Io)?;
    if !output.status.success() {
        return Err(RpfError::ExtractFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(temp)
}

/// Result of expanding a list of input paths: RPF9 archives are replaced
/// by their extracted temporary directories (kept alive in `_temp_dirs`).
#[derive(Debug)]
pub struct ExpandedInputs {
    pub paths: Vec<PathBuf>,
    pub _temp_dirs: Vec<tempfile::TempDir>,
    pub rpf9_detected: usize,
    pub rpf9_extracted: usize,
    pub rpf9_unavailable: usize,
}

/// Expand RPF9 archives in `paths` into temporary directories.
///
/// Never fails: unavailable extractors produce a note and leave the archive
/// in place so the caller can fall back to its existing behaviour.
pub fn preprocess(paths: Vec<PathBuf>) -> ExpandedInputs {
    let mut out_paths = Vec::with_capacity(paths.len());
    let mut temp_dirs = Vec::new();
    let mut rpf9_detected = 0usize;
    let mut rpf9_extracted = 0usize;
    let mut rpf9_unavailable = 0usize;

    for path in paths {
        if path.is_file() && is_rpf9(&path) {
            rpf9_detected += 1;
            match extract_to_temp(&path) {
                Ok(temp) => {
                    out_paths.push(temp.path().to_path_buf());
                    temp_dirs.push(temp);
                    rpf9_extracted += 1;
                }
                Err(RpfError::ToolNotAvailable) => {
                    eprintln!(
                        "note         RPF9 archive {} requires extraction; rage-tools-vi not found in PATH. \
                         Extract manually or install rage-tools-vi.",
                        path.display()
                    );
                    out_paths.push(path);
                    rpf9_unavailable += 1;
                }
                Err(e) => {
                    eprintln!("note         failed to extract {}: {e}", path.display());
                    out_paths.push(path);
                }
            }
        } else {
            out_paths.push(path);
        }
    }

    ExpandedInputs {
        paths: out_paths,
        _temp_dirs: temp_dirs,
        rpf9_detected,
        rpf9_extracted,
        rpf9_unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rpf9_magic() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("a.rpf");
        std::fs::write(&p1, b"RPF9rest").unwrap();
        assert!(is_rpf9(&p1));
        let p2 = dir.path().join("b.rpf");
        std::fs::write(&p2, b"9FPRrest").unwrap();
        assert!(is_rpf9(&p2));
        let p3 = dir.path().join("c.bin");
        std::fs::write(&p3, b"not rpf").unwrap();
        assert!(!is_rpf9(&p3));
        let p4 = dir.path().join("empty");
        std::fs::write(&p4, b"").unwrap();
        assert!(!is_rpf9(&p4));
    }

    #[test]
    fn preprocess_keeps_non_rpf_paths() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("shader.bin");
        std::fs::write(&p, b"not rpf").unwrap();
        let exp = preprocess(vec![p.clone()]);
        assert_eq!(exp.paths, vec![p]);
        assert_eq!(exp.rpf9_detected, 0);
        assert_eq!(exp.rpf9_extracted, 0);
    }

    #[test]
    fn extractor_reports_unavailable_when_missing() {
        if extractor_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.rpf");
        std::fs::write(&p, b"RPF9rest").unwrap();
        assert!(matches!(
            extract_to_temp(&p),
            Err(RpfError::ToolNotAvailable)
        ));
        let exp = preprocess(vec![p]);
        assert_eq!(exp.rpf9_detected, 1);
        assert_eq!(exp.rpf9_extracted, 0);
        assert_eq!(exp.rpf9_unavailable, 1);
    }
}
