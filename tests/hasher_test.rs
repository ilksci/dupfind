use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use dupfind::hasher;
use dupfind::hasher::algorithms::Blake3Algo;
use dupfind::FileInfo;

fn setup_temp_files(files: &[(&str, &[u8])]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "dupfind_hash_test_{:08x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content).unwrap();
    }
    dir
}

#[test]
fn test_find_duplicates() {
    let dir = setup_temp_files(&[
        ("a.txt", b"hello world"),
        ("b.txt", b"hello world"),
        ("c.txt", b"different!"),
        ("d.txt", b"hello world"),
    ]);

    let files = vec![
        FileInfo::new(dir.join("a.txt"), 11, Some(SystemTime::now()), false),
        FileInfo::new(dir.join("b.txt"), 11, Some(SystemTime::now()), false),
        FileInfo::new(dir.join("c.txt"), 10, Some(SystemTime::now()), false),
        FileInfo::new(dir.join("d.txt"), 11, Some(SystemTime::now()), false),
    ];

    let algo = Blake3Algo;
    let groups = hasher::find_duplicates(files, &algo).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].files.len(), 3);
    assert_eq!(groups[0].size, 11);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_no_duplicates() {
    let dir = setup_temp_files(&[("x.txt", b"aaaa"), ("y.txt", b"bbbb"), ("z.txt", b"cccc")]);

    let files = vec![
        FileInfo::new(dir.join("x.txt"), 4, None, false),
        FileInfo::new(dir.join("y.txt"), 4, None, false),
        FileInfo::new(dir.join("z.txt"), 4, None, false),
    ];

    let algo = Blake3Algo;
    let groups = hasher::find_duplicates(files, &algo).unwrap();
    assert!(groups.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_unique_size_dropped() {
    let dir = setup_temp_files(&[
        ("big.txt", b"this is a big file with more content"),
        ("small.txt", b"tiny"),
    ]);

    let files = vec![
        FileInfo::new(dir.join("big.txt"), 37, None, false),
        FileInfo::new(dir.join("small.txt"), 4, None, false),
    ];

    let algo = Blake3Algo;
    let groups = hasher::find_duplicates(files, &algo).unwrap();
    assert!(groups.is_empty());

    let _ = fs::remove_dir_all(&dir);
}
