pub mod algorithms;
pub mod parallel;

use std::collections::HashMap;

use serde::Serialize;

use crate::error::Result;
use crate::scanner::FileInfo;
use algorithms::HashAlgorithm;
use parallel::hash_files;

/// 重复文件组
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    /// SHA-256 或 BLAKE3 的十六进制哈希值
    pub hash: String,
    /// 每个文件的大小（组内文件大小相同）
    pub size: u64,
    /// 重复文件列表（≥ 2 个）
    pub files: Vec<FileInfo>,
}

/// 三级去重策略:
///
/// 1. **大小分桶** — 唯一大小的文件不可能重复，直接排除
/// 2. **前缀哈希** — 对同大小的文件只读首 4 KiB 做哈希，快速排除大量不同文件
/// 3. **完整哈希** — 前缀碰撞的文件再计算完整哈希确认
pub fn find_duplicates(
    files: Vec<FileInfo>,
    algo: &dyn HashAlgorithm,
) -> Result<Vec<DuplicateGroup>> {
    let file_count = files.len();

    // ── 阶段 1: 按大小分桶，丢弃唯一大小 ──────────────────
    let mut size_buckets: HashMap<u64, Vec<FileInfo>> = HashMap::new();
    for f in files {
        size_buckets.entry(f.size).or_default().push(f);
    }

    let mut candidates: Vec<FileInfo> = size_buckets
        .into_iter()
        .filter(|(_, b)| b.len() >= 2)
        .flat_map(|(_, b)| b)
        .collect();

    log::info!(
        "阶段1 大小分桶: {} → {} 个候选文件",
        file_count,
        candidates.len()
    );

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // ── 阶段 2: 前缀哈希（首 4 KiB）快速筛选 ──────────────
    if candidates.len() >= 2 {
        candidates = prefix_hash_filter(candidates)?;
        log::info!("阶段2 前缀哈希: 剩余 {} 个候选文件", candidates.len());
    }

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // ── 阶段 3: 完整哈希 + 分组 ──────────────────────────
    let hashed = hash_files(candidates, algo);

    let mut hash_buckets: HashMap<String, Vec<FileInfo>> = HashMap::new();
    for f in hashed {
        if let Some(ref hash) = f.hash {
            hash_buckets.entry(hash.clone()).or_default().push(f);
        }
    }

    let groups: Vec<DuplicateGroup> = hash_buckets
        .into_iter()
        .filter(|(_, b)| b.len() >= 2)
        .map(|(hash, bucket)| {
            let size = bucket.first().map(|f| f.size).unwrap_or(0);
            DuplicateGroup {
                hash,
                size,
                files: bucket,
            }
        })
        .collect();

    log::info!("阶段3 完整哈希: {} 个重复组", groups.len());
    Ok(groups)
}

/// 前缀哈希过滤：每个文件只读前 4 KiB 做哈希，碰撞的才进入完整哈希
fn prefix_hash_filter(files: Vec<FileInfo>) -> Result<Vec<FileInfo>> {
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::Read;

    let mut buckets: HashMap<String, Vec<FileInfo>> = HashMap::new();

    for mut f in files {
        let mut file = match fs::File::open(&f.path) {
            Ok(file) => file,
            Err(_) => continue,
        };

        let mut hasher = Sha256::new();
        let mut buf = [0u8; 4096];
        if let Ok(n) = file.read(&mut buf) {
            hasher.update(&buf[..n]);
            f.hash = Some(format!("{:x}", hasher.finalize()));
            if let Some(ref h) = f.hash {
                buckets.entry(h.clone()).or_default().push(f);
            }
        }
    }

    // 丢弃前缀哈希唯一的文件
    Ok(buckets
        .into_iter()
        .filter(|(_, b)| b.len() >= 2)
        .flat_map(|(_, b)| b)
        .collect())
}
