pub mod filter;

use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;
use walkdir::WalkDir;

use crate::cli::CliArgs;
use crate::error::Result;
use filter::FilterConfig;

/// 文件元信息
#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间
    #[serde(skip)]
    pub modified: Option<SystemTime>,
    /// 是否为符号链接
    #[serde(skip)]
    pub is_symlink: bool,
    /// 文件内容哈希（扫描后由哈希模块填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64, modified: Option<SystemTime>, is_symlink: bool) -> Self {
        Self {
            path,
            size,
            modified,
            is_symlink,
            hash: None,
        }
    }

    /// 文件是否为空
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// 扫描结果汇总
pub struct ScanSummary {
    pub total_files: usize,
    pub empty_files: usize,
    pub symlinks: usize,
}

/// 递归扫描目录，收集通过过滤器的文件
pub fn scan(args: &CliArgs) -> Result<(Vec<FileInfo>, ScanSummary)> {
    let filter = FilterConfig {
        min_size: args.min_size,
        extensions: args.extensions.iter().map(|e| e.to_lowercase()).collect(),
        exclude_patterns: args.exclude.clone(),
    };

    let mut files: Vec<FileInfo> = Vec::new();
    let mut empty_count = 0usize;
    let mut symlink_count = 0usize;

    for entry in WalkDir::new(&args.path)
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
            continue;
        }

        let metadata = entry.metadata()?;
        let size = metadata.len();
        let modified = metadata.modified().ok();

        if size == 0 {
            empty_count += 1;
        }

        if filter.matches(entry.path(), size) {
            files.push(FileInfo::new(
                entry.path().to_path_buf(),
                size,
                modified,
                false,
            ));
        }
    }

    let summary = ScanSummary {
        total_files: files.len(),
        empty_files: empty_count,
        symlinks: symlink_count,
    };

    log::info!(
        "扫描完成: {} 文件, {} 空文件, {} 符号链接",
        summary.total_files,
        summary.empty_files,
        summary.symlinks,
    );

    Ok((files, summary))
}
