use std::fs;
use std::path::PathBuf;

use dupfind::hasher::DuplicateGroup;
use dupfind::reporter;
use dupfind::scanner::FileInfo;

#[test]
fn test_json_report() {
    let group = DuplicateGroup {
        hash: "abc123".into(),
        size: 100,
        files: vec![
            FileInfo::new(PathBuf::from("/tmp/a.txt"), 100, None),
            FileInfo::new(PathBuf::from("/tmp/b.txt"), 100, None),
        ],
    };

    let dir = std::env::temp_dir().join(format!(
        "dupfind_reporter_{:08x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
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
            FileInfo::new(PathBuf::from("/tmp/x.jpg"), 200, None),
            FileInfo::new(PathBuf::from("/tmp/y.jpg"), 200, None),
        ],
    };

    let dir = std::env::temp_dir().join(format!(
        "dupfind_reporter_{:08x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            + 1
    ));
    fs::create_dir_all(&dir).unwrap();

    let output = dir.join("report.csv");
    reporter::export(&[group], &output).unwrap();

    let content = fs::read_to_string(&output).unwrap();
    assert!(content.contains("def456"));
    assert!(content.contains("x.jpg"));
    assert!(content.contains("y.jpg"));
    // Should have header + 2 data rows.
    assert_eq!(content.lines().count(), 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_unsupported_format() {
    let group = DuplicateGroup {
        hash: "000".into(),
        size: 1,
        files: vec![FileInfo::new(PathBuf::from("a"), 1, None)],
    };

    let output = PathBuf::from("report.txt");
    let result = reporter::export(&[group], &output);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported report format"));
}
