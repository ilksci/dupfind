use std::path::PathBuf;
use std::result::Result as StdResult;

use clap::{Parser, ValueEnum};

use crate::cleaner::KeepStrategy;

/// 高性能重复文件查找与清理工具
#[derive(Parser, Debug)]
#[command(name = "dupfind", version, about = "查找并清理重复文件", long_about = None)]
pub struct CliArgs {
    /// 要扫描的目录路径
    #[arg(
        short = 'p',
        long = "path",
        default_value = ".",
        value_hint = clap::ValueHint::DirPath
    )]
    pub path: PathBuf,

    /// 最小文件大小，小于此值的文件将被忽略（如 "1MB", "500KB"）
    #[arg(short = 'm', long = "min-size", value_parser = parse_size)]
    pub min_size: Option<u64>,

    /// 只扫描指定扩展名的文件，逗号分隔（如 "jpg,png,mp4"）
    #[arg(short = 'e', long = "ext", value_delimiter = ',')]
    pub extensions: Vec<String>,

    /// 排除路径中包含这些字符串的文件/目录
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,

    /// 导出重复文件报告（支持 .json / .csv / .html）
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// 删除策略
    #[arg(short = 'd', long = "delete", value_enum)]
    pub delete: Option<DeleteStrategyArg>,

    /// 安全预览模式：只显示将要删除的文件，不实际删除
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// 移动到系统回收站而非永久删除
    #[arg(long = "trash")]
    pub use_trash: bool,

    /// 以表格形式在终端直接打印重复文件组
    #[arg(short = 't', long = "table")]
    pub table: bool,

    /// 哈希算法选择
    #[arg(long = "hash-algo", value_enum, default_value_t = HashAlgoArg::Blake3)]
    pub hash_algo: HashAlgoArg,

    /// 配置文件路径（默认为当前目录 .dupfind.toml）
    #[arg(long = "config")]
    pub config: Option<PathBuf>,

    /// 详细输出级别（-v 或 -vv）
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// CLI 删除策略枚举
#[derive(ValueEnum, Clone, Debug)]
pub enum DeleteStrategyArg {
    /// 交互式逐组选择
    Interactive,
    /// 自动保留每组中修改时间最新的文件
    KeepNewest,
    /// 自动保留每组中修改时间最早的文件
    KeepOldest,
    /// 自动保留每组中路径最短的文件
    KeepShortest,
    /// 自动保留每组中体积最大的文件
    KeepLargest,
    /// 自动保留每组中体积最小的文件
    KeepSmallest,
}

impl From<DeleteStrategyArg> for KeepStrategy {
    fn from(arg: DeleteStrategyArg) -> Self {
        match arg {
            DeleteStrategyArg::Interactive => KeepStrategy::Interactive,
            DeleteStrategyArg::KeepNewest => KeepStrategy::Newest,
            DeleteStrategyArg::KeepOldest => KeepStrategy::Oldest,
            DeleteStrategyArg::KeepShortest => KeepStrategy::Shortest,
            DeleteStrategyArg::KeepLargest => KeepStrategy::Largest,
            DeleteStrategyArg::KeepSmallest => KeepStrategy::Smallest,
        }
    }
}

/// CLI 哈希算法枚举
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum HashAlgoArg {
    /// BLAKE3 — 速度更快，适合大文件（默认）
    #[default]
    Blake3,
    /// SHA-256 — 密码学安全，兼容性更好
    Sha256,
}

/// 解析命令行参数
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

/// 解析人类可读的文件大小（如 "1MB", "500KB", "2GB"）
///
/// 支持小数（如 "1.5GB"）。支持的后缀（不区分大小写）：
///   B, KB, KiB, MB, MiB, GB, GiB, TB, TiB
/// 不带后缀的值视为字节数。
pub fn parse_size(raw: &str) -> StdResult<u64, String> {
    let raw = raw.trim().to_uppercase();

    let (num_str, multiplier): (&str, f64) = {
        if let Some(rest) = raw.strip_suffix("TIB") {
            (rest.trim(), 1_099_511_627_776.0)
        } else if let Some(rest) = raw.strip_suffix("GIB") {
            (rest.trim(), 1_073_741_824.0)
        } else if let Some(rest) = raw.strip_suffix("MIB") {
            (rest.trim(), 1_048_576.0)
        } else if let Some(rest) = raw.strip_suffix("KIB") {
            (rest.trim(), 1_024.0)
        } else if let Some(rest) = raw.strip_suffix("TB") {
            (rest.trim(), 1_000_000_000_000.0)
        } else if let Some(rest) = raw.strip_suffix("GB") {
            (rest.trim(), 1_000_000_000.0)
        } else if let Some(rest) = raw.strip_suffix("MB") {
            (rest.trim(), 1_000_000.0)
        } else if let Some(rest) = raw.strip_suffix("KB") {
            (rest.trim(), 1_000.0)
        } else if let Some(rest) = raw.strip_suffix('B') {
            (rest.trim(), 1.0)
        } else {
            (raw.trim(), 1.0)
        }
    };

    let num: f64 = num_str
        .parse()
        .map_err(|e| format!("无效的数值 '{}': {}", num_str, e))?;

    let result = num * multiplier;
    if result > u64::MAX as f64 || result < 0.0 {
        return Err(format!("数值过大 '{}'", raw));
    }

    Ok(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_bytes() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("100B").unwrap(), 100);
    }

    #[test]
    fn test_parse_size_kb() {
        assert_eq!(parse_size("1KB").unwrap(), 1_000);
        assert_eq!(parse_size("2kb").unwrap(), 2_000);
    }

    #[test]
    fn test_parse_size_mb() {
        assert_eq!(parse_size("1MB").unwrap(), 1_000_000);
        assert_eq!(parse_size("5.5MB").unwrap(), 5_500_000);
    }

    #[test]
    fn test_parse_size_gb() {
        assert_eq!(parse_size("1GB").unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_parse_size_kib() {
        assert_eq!(parse_size("1KIB").unwrap(), 1_024);
    }

    #[test]
    fn test_parse_size_mib() {
        assert_eq!(parse_size("1MIB").unwrap(), 1_048_576);
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("abc").is_err());
        assert!(parse_size("").is_err());
    }
}
