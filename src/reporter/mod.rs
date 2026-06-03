pub mod csv;
pub mod json;

use std::path::Path;

use crate::error::{DupfindError, Result};
use crate::hasher::DuplicateGroup;

/// Dispatch to the correct reporter based on the output file extension.
///
/// - `.json` → JSON reporter
/// - `.csv`  → CSV reporter
/// - anything else → error
pub fn export(groups: &[DuplicateGroup], output: &Path) -> Result<()> {
    match output.extension().and_then(|e| e.to_str()) {
        Some("json") => json::JsonReporter.write(groups, output),
        Some("csv") => csv::CsvReporter.write(groups, output),
        Some(ext) => Err(DupfindError::Other(format!(
            "Unsupported report format '.{}'. Use '.json' or '.csv'.",
            ext
        ))),
        None => Err(DupfindError::Other(
            "Output file must have a .json or .csv extension.".into(),
        )),
    }
}

/// Trait for exporting duplicate reports.
pub trait Reporter {
    /// Serialise `groups` and write the result to `output`.
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()>;
}
