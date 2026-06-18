use std::fs;
use std::path::PathBuf;

use dupfind::reporter;
use dupfind::{DuplicateGroup, FileInfo};

fn temp_dir(prefix: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    std::env::temp_dir().join(format!("dupfind_{}_{:08x}_{:04x}", prefix, nanos, seq))
}

#[test]
fn test_json_report() {
    let group = DuplicateGroup {
        hash: "abc123".into(),
        size: 100,
        files: vec![
            FileInfo::new(PathBuf::from("/tmp/a.txt"), 100, None, false),
            FileInfo::new(PathBuf::from("/tmp/b.txt"), 100, None, false),
        ],
    };

    let dir = temp_dir("reporter");
    fs::create_dir_all(&dir).unwrap();

    let output = dir.join("report.json");
    reporter::export(&[group], &output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("abc123"));
    assert!(content.contains("a.txt"));
    assert!(content.contains("b.txt"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_csv_report() {
    let group = DuplicateGroup {
        hash: "def456".into(),
        size: 200,
        files: vec![
            FileInfo::new(PathBuf::from("/tmp/x.jpg"), 200, None, false),
            FileInfo::new(PathBuf::from("/tmp/y.jpg"), 200, None, false),
        ],
    };

    let dir = temp_dir("reporter");
    fs::create_dir_all(&dir).unwrap();

    let output = dir.join("report.csv");
    reporter::export(&[group], &output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("def456"));
    assert!(content.contains("x.jpg"));
    assert!(content.contains("y.jpg"));
    assert_eq!(content.lines().count(), 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_html_report() {
    let group = DuplicateGroup {
        hash: "abc123def456".into(),
        size: 100,
        files: vec![
            FileInfo::new(PathBuf::from("/tmp/a.txt"), 100, None, false),
            FileInfo::new(PathBuf::from("/tmp/b.txt"), 100, None, false),
        ],
    };

    let dir = temp_dir("reporter");
    fs::create_dir_all(&dir).unwrap();

    let output = dir.join("report.html");
    reporter::export(&[group], &output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("<!DOCTYPE html>"));
    assert!(content.contains("abc123def456"));
    assert!(content.contains("a.txt"));
    assert!(content.contains("b.txt"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_unsupported_format() {
    let group = DuplicateGroup {
        hash: "000".into(),
        size: 1,
        files: vec![FileInfo::new(PathBuf::from("a"), 1, None, false)],
    };

    let result = reporter::export(&[group], &PathBuf::from("report.txt"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("不支持的报告格式"));
}
