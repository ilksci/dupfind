use std::fs;
use std::io::BufWriter;
use std::path::Path;

use crate::error::Result;
use crate::hasher::DuplicateGroup;
use crate::reporter::Reporter;

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()> {
        let file = fs::File::create(output)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, groups)?;
        Ok(())
    }
}
