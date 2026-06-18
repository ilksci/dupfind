//! dupfind v3 — 高性能重复文件查找与清理工具
//!
//! 本 crate 重新导出所有子 crate 的公开 API，
//! 供集成测试和基准测试使用。各子 crate 已重新导出
//! 核心类型以保持 `dupfind::module::Type` 的访问路径。

pub use dupfind_cleaner as cleaner;
pub use dupfind_cli as cli;
pub use dupfind_core as core;
pub use dupfind_hasher as hasher;
pub use dupfind_reporter as reporter;
pub use dupfind_scanner as scanner;

// 常用顶层类型重新导出
pub use dupfind_cleaner::{CleanOptions, KeepStrategy};
pub use dupfind_core::{
    format_bytes, DupfindError, DuplicateGroup, FileInfo, HashAlgorithm, Reporter, Result,
};
