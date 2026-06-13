//! 基于 tokio 的异步文件哈希计算。
//!
//! 使用 `tokio::fs::read` 异步文件读取，CPU 哈希在当前任务中同步执行。
//! 相比同步 IO 的优势：不阻塞 OS 线程等待磁盘 I/O，
//! 配合 rayon 的 `par_iter` 可实现混合并行。

use std::io;

use dupfind_core::{FileInfo, HashAlgorithm};

/// 使用 tokio 异步文件读取计算文件哈希
///
/// 每个文件异步读取全部内容到内存，然后同步计算哈希。
/// 外层可配合多个 tokio task 实现并发 I/O。
pub async fn hash_files_async(
    files: Vec<FileInfo>,
    algo: &(dyn HashAlgorithm + Sync),
) -> Vec<FileInfo> {
    let total = files.len();
    log::info!("异步哈希计算: {} 个文件...", total);

    let mut results = Vec::with_capacity(files.len());

    for mut f in files {
        // 异步读取文件内容
        match tokio::fs::read(&f.path).await {
            Ok(data) => {
                let mut cursor = io::Cursor::new(&data);
                f.hash = algo.hash(&mut cursor).ok();
            }
            Err(e) => {
                log::debug!("异步读取失败 {}: {}", f.path.display(), e);
            }
        }
        results.push(f);
    }

    log::info!("异步哈希完成: {} 个文件", results.len());
    results
}
