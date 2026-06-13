use std::io::{self, Read};
use std::path::Path;

use crate::error::Result;
use crate::types::DuplicateGroup;

/// 文件哈希算法抽象 trait
///
/// v3 可扩展更多算法（如 XXHash、MD5 等）
pub trait HashAlgorithm: Send + Sync {
    /// 对 reader 内容计算哈希，返回十六进制字符串
    fn hash(&self, reader: &mut dyn Read) -> io::Result<String>;
    /// 算法名称
    fn name(&self) -> &'static str;
}

/// 报告导出 trait
pub trait Reporter {
    /// 将重复文件组序列化并写入 output
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}
