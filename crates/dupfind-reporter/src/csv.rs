use std::path::Path;

use dupfind_core::error::Result;
use dupfind_core::{DuplicateGroup, Reporter};

pub struct CsvReporter;

impl Reporter for CsvReporter {
    fn write(&self, groups: &[DuplicateGroup], output: &Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(output)?;

        // Header row.
        wtr.write_record(["group_index", "hash", "size", "file_path"])?;

        for (gi, group) in groups.iter().enumerate() {
            for file in &group.files {
                wtr.write_record(&[
                    gi.to_string(),
                    group.hash.clone(),
                    group.size.to_string(),
                    file.path.to_string_lossy().to_string(),
                ])?;
            }
        }

        wtr.flush()?;
        Ok(())
    }
}
