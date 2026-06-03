use std::ffi::OsStr;
use std::path::Path;

/// Filtering rules applied during directory scanning.
#[derive(Debug, Clone)]
pub struct FilterConfig {
    /// Minimum file size in bytes (files smaller than this are skipped).
    pub min_size: Option<u64>,

    /// Allowed extensions (lowercase, without leading dot).
    /// An empty vec means all extensions are allowed.
    pub extensions: Vec<String>,

    /// Path fragments to exclude — any file whose path contains one of these
    /// strings is skipped.
    pub exclude_patterns: Vec<String>,
}

impl FilterConfig {
    /// Returns `true` when a file passes every active filter.
    pub fn matches(&self, path: &Path, size: u64) -> bool {
        // Size check.
        if let Some(min) = self.min_size {
            if size < min {
                return false;
            }
        }

        // Extension check.
        if !self.extensions.is_empty() {
            if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                let ext_lower = ext.to_lowercase();
                if !self.extensions.iter().any(|e| e.as_str() == ext_lower.as_str()) {
                    return false;
                }
            } else {
                // No extension at all → reject when extensions are specified.
                return false;
            }
        }

        // Exclude-pattern check.
        if !self.exclude_patterns.is_empty() {
            let path_str = path.to_string_lossy();
            if self
                .exclude_patterns
                .iter()
                .any(|pat| path_str.contains(pat.as_str()))
            {
                return false;
            }
        }

        true
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            min_size: None,
            extensions: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_size_filter() {
        let cfg = FilterConfig {
            min_size: Some(100),
            ..Default::default()
        };
        assert!(cfg.matches(Path::new("a.txt"), 200));
        assert!(!cfg.matches(Path::new("a.txt"), 50));
    }

    #[test]
    fn test_extension_filter() {
        let cfg = FilterConfig {
            extensions: vec!["jpg".into(), "png".into()],
            ..Default::default()
        };
        assert!(cfg.matches(Path::new("photo.jpg"), 100));
        assert!(cfg.matches(Path::new("img.PNG"), 100));
        assert!(!cfg.matches(Path::new("doc.txt"), 100));
        assert!(!cfg.matches(Path::new("noext"), 100));
    }

    #[test]
    fn test_exclude_filter() {
        let cfg = FilterConfig {
            exclude_patterns: vec!["node_modules".into(), ".git".into()],
            ..Default::default()
        };
        assert!(!cfg.matches(PathBuf::from("proj/node_modules/pkg/file.js").as_path(), 100));
        assert!(cfg.matches(Path::new("src/main.rs"), 100));
    }
}
