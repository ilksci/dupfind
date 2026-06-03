use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::cleaner::KeepStrategy;

/// A fast duplicate file finder and cleaner.
#[derive(Parser, Debug)]
#[command(name = "dupfind", version, about = "Find and clean duplicate files", long_about = None)]
pub struct CliArgs {
    /// Directory path to scan for duplicates
    #[arg(
        short = 'p',
        long = "path",
        default_value = ".",
        value_hint = clap::ValueHint::DirPath
    )]
    pub path: PathBuf,

    /// Minimum file size to consider (e.g. "1MB", "500KB", "2GB")
    #[arg(short = 'm', long = "min-size", value_parser = parse_size)]
    pub min_size: Option<u64>,

    /// Comma-separated list of file extensions to include (e.g. "jpg,png,mp4")
    #[arg(short = 'e', long = "ext", value_delimiter = ',')]
    pub extensions: Vec<String>,

    /// Exclude paths matching these patterns
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,

    /// Export duplicate report to file (JSON or CSV determined by extension)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Delete strategy for handling duplicates
    #[arg(short = 'd', long = "delete", value_enum)]
    pub delete: Option<DeleteStrategyArg>,
}

/// CLI-facing delete strategy enum (maps to KeepStrategy).
#[derive(ValueEnum, Clone, Debug)]
pub enum DeleteStrategyArg {
    /// Interactively choose which files to keep
    Interactive,
    /// Automatically keep the newest file in each group
    KeepNewest,
    /// Automatically keep the oldest file in each group
    KeepOldest,
    /// Automatically keep the file with the shortest path
    KeepShortest,
}

impl From<DeleteStrategyArg> for KeepStrategy {
    fn from(arg: DeleteStrategyArg) -> Self {
        match arg {
            DeleteStrategyArg::Interactive => KeepStrategy::Interactive,
            DeleteStrategyArg::KeepNewest => KeepStrategy::Newest,
            DeleteStrategyArg::KeepOldest => KeepStrategy::Oldest,
            DeleteStrategyArg::KeepShortest => KeepStrategy::Shortest,
        }
    }
}

/// Parse the CLI arguments.
pub fn parse_args() -> CliArgs {
    CliArgs::parse()
}

/// Parse human-readable file sizes like "1MB", "500KB", "2GB".
///
/// Supports fractional values (e.g. "1.5GB" = 1_500_000_000).
/// Recognised suffixes (case-insensitive):
///   B, KB, KiB, MB, MiB, GB, GiB, TB, TiB
///
/// Suffix-less values are treated as bytes.
fn parse_size(raw: &str) -> std::result::Result<u64, String> {
    let raw = raw.trim().to_uppercase();

    // Extract numeric part and optional suffix.
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
        .map_err(|e| format!("Invalid size value '{}': {}", num_str, e))?;

    let result = num * multiplier;
    if result > u64::MAX as f64 || result < 0.0 {
        return Err(format!("Size value '{}' is too large", raw));
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
