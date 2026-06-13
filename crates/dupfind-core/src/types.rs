use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;

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
    /// 检测到的文件类型（如 "JPEG image", "PDF document"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_type: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf, size: u64, modified: Option<SystemTime>, is_symlink: bool) -> Self {
        Self {
            path,
            size,
            modified,
            is_symlink,
            hash: None,
            detected_type: None,
        }
    }

    /// 文件是否为空
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

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

/// 人类可读的字节表示
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("TB", 1_099_511_627_776.0),
        ("GB", 1_073_741_824.0),
        ("MB", 1_048_576.0),
        ("KB", 1_024.0),
        ("B", 1.0),
    ];
    for (unit, div) in UNITS {
        let val = bytes as f64 / div;
        if val >= 1.0 || *unit == "B" {
            return format!("{:.1} {}", val, unit);
        }
    }
    format!("{bytes} B")
}
