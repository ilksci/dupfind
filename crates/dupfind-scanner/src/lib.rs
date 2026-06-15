pub mod filter;

use std::collections::HashMap;
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

use dupfind_core::error::Result;
use filter::FilterConfig;

// 重导出核心类型以保持向后兼容（同时供本 crate 内部使用）
pub use dupfind_core::FileInfo;

/// 扫描配置（由 CLI 层传入）
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 要扫描的目录路径
    pub path: PathBuf,
    /// 最小文件大小（字节）
    pub min_size: Option<u64>,
    /// 允许的扩展名（小写，不含点）
    pub extensions: Vec<String>,
    /// 路径排除模式
    pub exclude_patterns: Vec<String>,
    /// 文件类型过滤器（如 "image", "video", "document"）
    pub type_filter: Vec<String>,
}

/// 扫描结果汇总
pub struct ScanSummary {
    pub total_files: usize,
    pub empty_files: usize,
    pub symlinks: usize,
    /// 文件类型分布统计
    pub type_distribution: HashMap<String, usize>,
}

/// 递归扫描目录，收集通过过滤器的文件
pub fn scan(config: &ScanConfig) -> Result<(Vec<FileInfo>, ScanSummary)> {
    let filter = FilterConfig {
        min_size: config.min_size,
        extensions: config.extensions.iter().map(|e| e.to_lowercase()).collect(),
        exclude_patterns: config.exclude_patterns.clone(),
        type_filter: if config.type_filter.is_empty() {
            None
        } else {
            Some(config.type_filter.clone())
        },
    };

    let mut files: Vec<FileInfo> = Vec::new();
    let mut empty_count = 0usize;
    let mut symlink_count = 0usize;
    let mut skipped_count = 0u64;
    let mut type_distribution: HashMap<String, usize> = HashMap::new();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} 正在扫描目录 [{elapsed_precise}] 已发现 {pos} 个文件（跳过 {msg} 个）",
            )
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    for entry in WalkDir::new(&config.path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
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

        // walkdir 缓存的元数据
        let file_type = entry.file_type();
        let is_symlink = file_type.is_symlink();

        if is_symlink {
            symlink_count += 1;
            log::debug!("检测到符号链接: {}", entry.path().display());
        }

        // 只处理普通文件（符号链接不跟进）
        if !file_type.is_file() || is_symlink {
            skipped_count += 1;
            continue;
        }

        let metadata = entry.metadata()?;
        let size = metadata.len();
        let modified = metadata.modified().ok();

        if size == 0 {
            empty_count += 1;
        }

        // 魔术字节类型检测
        let detected_type = detect_file_type(entry.path());

        // 统计类型分布
        if let Some(ref dt) = detected_type {
            *type_distribution.entry(dt.clone()).or_default() += 1;
        } else {
            *type_distribution.entry("unknown".into()).or_default() += 1;
        }

        if filter.matches(entry.path(), size, detected_type.as_deref()) {
            let mut file_info = FileInfo::new(
                entry.path().to_path_buf(),
                size,
                modified,
                false,
            );
            file_info.detected_type = detected_type;
            files.push(file_info);
            pb.set_position(files.len() as u64);
            pb.set_message(skipped_count.to_string());
        } else {
            skipped_count += 1;
        }
    }

    pb.finish_and_clear();

    let summary = ScanSummary {
        total_files: files.len(),
        empty_files: empty_count,
        symlinks: symlink_count,
        type_distribution,
    };

    log::info!(
        "扫描完成: {} 文件, {} 空文件, {} 符号链接",
        summary.total_files,
        summary.empty_files,
        summary.symlinks,
    );

    Ok((files, summary))
}

/// 检测文件的魔术字节类型
fn detect_file_type(path: &std::path::Path) -> Option<String> {
    use std::fs;
    use std::io::Read;

    let mut file = fs::File::open(path).ok()?;
    let mut buf = [0u8; 8192];
    let n = file.read(&mut buf).ok()?;
    let kind = infer::get(&buf[..n])?;
    Some(format!(
        "{} ({})",
        kind.mime_type(),
        kind.extension()
    ))
}
