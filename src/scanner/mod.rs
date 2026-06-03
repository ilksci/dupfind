pub mod filter;

use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;
use walkdir::WalkDir;

use crate::cli::CliArgs;
use crate::error::Result;
use filter::FilterConfig;

/// Metadata for a scanned file.
#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    /// Absolute or relative path to the file.
    pub path: PathBuf,
    /// File size in bytes.
    pub size: u64,
    /// Last modification time (if available).
    #[serde(skip)]
    pub modified: Option<SystemTime>,
    /// SHA-256 hash — filled in later by the hasher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64, modified: Option<SystemTime>) -> Self {
        Self {
            path,
            size,
            modified,
            hash: None,
        }
    }
}

/// Recursively scan `root` and return every file that passes the filter.
pub fn scan(args: &CliArgs) -> Result<Vec<FileInfo>> {
    let filter = FilterConfig {
        min_size: args.min_size,
        extensions: args.extensions.iter().map(|e| e.to_lowercase()).collect(),
        exclude_patterns: args.exclude.clone(),
    };

    let mut files: Vec<FileInfo> = Vec::new();

    for entry in WalkDir::new(&args.path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip directories that match exclude patterns so we don't
            // descend into them at all.
            if filter.exclude_patterns.is_empty() {
                return true;
            }
            let path_str = e.path().to_string_lossy();
            !filter
                .exclude_patterns
                .iter()
                .any(|pat| path_str.contains(pat.as_str()))
        })
    {
        let entry = entry?;

        // We only care about regular files.
        if !entry.file_type().is_file() {
            continue;
        }

        // walkdir metadata is cached in `entry` — cheap.
        let metadata = entry.metadata()?;
        let size = metadata.len();
        let modified = metadata.modified().ok();

        if filter.matches(entry.path(), size) {
            files.push(FileInfo::new(entry.path().to_path_buf(), size, modified));
        }
    }

    Ok(files)
}
