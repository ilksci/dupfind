pub mod csv;
pub mod html;
pub mod json;

use std::path::Path;

use crate::error::{DupfindError, Result};
use crate::hasher::DuplicateGroup;

/// 根据输出文件的扩展名调度到对应的报告器
///
/// - `.json` → JSON
/// - `.csv`  → CSV
/// - `.html` → HTML
pub fn export(groups: &[DuplicateGroup], output: &Path) -> Result<()> {
    match output.extension().and_then(|e| e.to_str()) {
        Some("json") => json::JsonReporter.write(groups, output),
        Some("csv") => csv::CsvReporter.write(groups, output),
        Some("html") | Some("htm") => html::HtmlReporter.write(groups, output),
        Some(ext) => Err(DupfindError::Other(format!(
            "不支持的报告格式 '.{}'，请使用 .json / .csv / .html",
            ext
        ))),
        None => Err(DupfindError::Other(
            "输出文件必须有扩展名（.json / .csv / .html）".into(),
        )),
    }
}

/// 报告导出 trait
pub trait Reporter {
    /// 将重复文件组序列化并写入 output
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}
