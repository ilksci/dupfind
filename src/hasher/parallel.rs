use std::fs;
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use super::algorithms::HashAlgorithm;
use crate::scanner::FileInfo;

/// 使用 rayon 并行计算文件哈希（简单模式）
pub fn hash_files(files: Vec<FileInfo>, algo: &dyn HashAlgorithm) -> Vec<FileInfo> {
    let total = files.len() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} 哈希计算 [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}, 预计 {eta})")
            .unwrap()
            .progress_chars("━╸ "),
    );

    let results: Vec<FileInfo> = files
        .into_par_iter()
        .map(|mut f| {
            f.hash = compute_hash(&f.path, algo).ok();
            pb.inc(1);
            f
        })
        .collect();

    pb.finish_and_clear();
    results
}

/// 基于 channel 的流水线哈希（演示 thread + mpsc + Arc<Mutex> 并发模式）
///
/// 工作窃取模型：多个工作线程从共享队列中取文件、计算哈希、通过 channel 返回结果。
/// 这是 rayon 之外的另一种并行策略，v3 可扩展为 tokio 异步 IO。
#[allow(dead_code)]
pub fn hash_files_pipeline(files: Vec<FileInfo>, algo: Arc<dyn HashAlgorithm>) -> Vec<FileInfo> {
    if files.is_empty() {
        return vec![];
    }

    let total = files.len() as u64;
    let num_workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let queue: Arc<Mutex<Vec<FileInfo>>> = Arc::new(Mutex::new(files));
    let (tx, rx) = mpsc::channel::<FileInfo>();

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} 流水线哈希 [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}",
            )
            .unwrap()
            .progress_chars("━╸ "),
    );

    // 启动工作线程
    let mut handles = vec![];
    for _ in 0..num_workers {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        let algo = Arc::clone(&algo);
        handles.push(thread::spawn(move || {
            loop {
                let file = {
                    let mut q = queue.lock().unwrap();
                    q.pop()
                };
                match file {
                    Some(mut f) => {
                        f.hash = compute_hash(&f.path, algo.as_ref()).ok();
                        // 发送失败说明接收端已关闭，退出
                        if tx.send(f).is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }));
    }

    // 生产者线程结束，关闭发送端
    drop(tx);

    // 主线程收集结果
    let mut results = Vec::with_capacity(total as usize);
    for f in rx {
        pb.inc(1);
        results.push(f);
    }

    pb.finish_and_clear();

    for h in handles {
        let _ = h.join();
    }

    results
}

/// 打开文件并计算哈希
fn compute_hash(path: &Path, algo: &dyn HashAlgorithm) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(128 * 1024, file);
    algo.hash(&mut reader)
}
