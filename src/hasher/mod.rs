pub mod parallel;

use std::collections::HashMap;

use serde::Serialize;

use crate::error::Result;
use crate::scanner::FileInfo;

/// A group of files that share the same content hash (i.e. duplicates).
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    /// The SHA-256 hex digest shared by all files in this group.
    pub hash: String,
    /// The byte-size of each file in this group.
    pub size: u64,
    /// The duplicate files (always ≥ 2 entries).
    pub files: Vec<FileInfo>,
}

/// Two-phase duplicate detection:
///
/// 1. **Size bucket** — files with a unique size cannot have duplicates; drop them.
/// 2. **Hash** — compute SHA-256 for every file that shares its size with at least
///    one other file, then group by hash.
pub fn find_duplicates(mut files: Vec<FileInfo>) -> Result<Vec<DuplicateGroup>> {
    // ── Phase 1: group by size, drop singletons ──────────────────────
    let mut size_buckets: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    for f in files.drain(..) {
        size_buckets.entry(f.size).or_default().push(f);
    }

    // Collect only the files that share a size with ≥ 1 other file.
    let candidates: Vec<FileInfo> = size_buckets
        .into_iter()
        .filter(|(_, bucket)| bucket.len() >= 2)
        .flat_map(|(_, bucket)| bucket)
        .collect();

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // ── Phase 2: compute hashes (parallel), group by hash ────────────
    let hashed = parallel::hash_files(candidates);

    let mut hash_buckets: HashMap<String, Vec<FileInfo>> = HashMap::new();
    for f in hashed {
        if let Some(ref hash) = f.hash {
            hash_buckets.entry(hash.clone()).or_default().push(f);
        }
    }

    // Keep only groups with ≥ 2 files that have the same hash.
    let groups: Vec<DuplicateGroup> = hash_buckets
        .into_iter()
        .filter(|(_, bucket)| bucket.len() >= 2)
        .map(|(hash, bucket)| {
            // All files in a bucket share the same size because we pre-filtered.
            let size = bucket.first().map(|f| f.size).unwrap_or(0);
            DuplicateGroup {
                hash,
                size,
                files: bucket,
            }
        })
        .collect();

    Ok(groups)
}
