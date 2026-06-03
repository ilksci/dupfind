pub mod interactive;

use std::fs;
use std::time::SystemTime;

use crate::error::Result;
use crate::hasher::DuplicateGroup;

/// Strategy for deciding which file(s) to keep in a duplicate group.
#[derive(Debug, Clone)]
pub enum KeepStrategy {
    /// Keep the file with the most recent modification time.
    Newest,
    /// Keep the file with the oldest modification time.
    Oldest,
    /// Keep the file with the shortest path name.
    Shortest,
    /// Let the user decide interactively (TUI).
    Interactive,
}

/// Remove duplicate files according to the chosen strategy.
pub fn clean(groups: &[DuplicateGroup], strategy: &KeepStrategy) -> Result<()> {
    match strategy {
        KeepStrategy::Interactive => interactive::run(groups),
        other => clean_auto(groups, other),
    }
}

/// Non-interactive clean: apply the strategy to pick one keeper per group,
/// delete the rest.
fn clean_auto(groups: &[DuplicateGroup], strategy: &KeepStrategy) -> Result<()> {
    let mut total_deleted: usize = 0;
    let mut total_bytes: u64 = 0;

    for group in groups {
        let keeper_idx = choose_keeper(&group.files, strategy);
        let to_delete: Vec<_> = group
            .files
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != keeper_idx)
            .collect();

        for (_, file) in &to_delete {
            match fs::remove_file(&file.path) {
                Ok(()) => {
                    println!("  Deleted: {}", file.path.display());
                    total_deleted += 1;
                    total_bytes += file.size;
                }
                Err(e) => {
                    eprintln!(
                        "  [WARN] Failed to delete {}: {}",
                        file.path.display(),
                        e
                    );
                }
            }
        }
    }

    println!(
        "Done. Deleted {} files, freed {} bytes.",
        total_deleted, total_bytes
    );
    Ok(())
}

/// Pick the index of the file to keep based on the strategy.
fn choose_keeper(files: &[crate::scanner::FileInfo], strategy: &KeepStrategy) -> usize {
    match strategy {
        KeepStrategy::Newest => files
            .iter()
            .enumerate()
            .max_by_key(|(_, f)| f.modified.unwrap_or(SystemTime::UNIX_EPOCH))
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Oldest => files
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| f.modified.unwrap_or(SystemTime::UNIX_EPOCH))
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Shortest => files
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| f.path.to_string_lossy().len())
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Interactive => {
            // Shouldn't be called — interactive is handled separately.
            0
        }
    }
}
