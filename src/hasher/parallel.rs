use std::fs;
use std::io::{self, Read};
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use sha2::{Digest, Sha256};

use crate::scanner::FileInfo;

/// Compute SHA-256 hashes for `files` in parallel using rayon.
///
/// A progress bar is shown on stderr while hashing.
/// Files that cannot be opened or read will have their `hash` field left as `None`
/// (they are effectively skipped).
pub fn hash_files(mut files: Vec<FileInfo>) -> Vec<FileInfo> {
    let total = files.len() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} Hashing [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}, ETA {eta})",
            )
            .unwrap()
            .progress_chars("━╸ "),
    );

    files
        .par_iter_mut()
        .for_each(|f| {
            f.hash = hash_file(&f.path).ok();
            pb.inc(1);
        });

    pb.finish_and_clear();
    files
}

/// Read a file and return the hex-encoded SHA-256 digest of its contents.
fn hash_file(path: &Path) -> io::Result<String> {
    const BUF_SIZE: usize = 128 * 1024; // 128 KiB read buffer

    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::with_capacity(BUF_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
