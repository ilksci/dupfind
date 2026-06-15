pub mod algorithms;
pub mod async_hash;
pub mod parallel;
pub mod similar;

use std::collections::HashMap;

use dupfind_core::error::Result;
use dupfind_core::{FileInfo, HashAlgorithm};
use parallel::hash_files;

// 重导出核心类型以保持向后兼容
pub use dupfind_core::DuplicateGroup;

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
    use indicatif::{ProgressBar, ProgressStyle};
    use sha2::{Digest, Sha256};
    use std::fs;
    use std::io::Read;

    let total = files.len() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} 前缀哈希筛选 [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("━╸ "),
    );

    let mut buckets: HashMap<String, Vec<FileInfo>> = HashMap::new();

    for mut f in files {
        pb.inc(1);
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

    pb.finish_and_clear();

    // 丢弃前缀哈希唯一的文件
    Ok(buckets
        .into_iter()
        .filter(|(_, b)| b.len() >= 2)
        .flat_map(|(_, b)| b)
        .collect())
}
