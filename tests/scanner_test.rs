use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dupfind::scanner::{self, ScanConfig};

/// 创建临时目录并写入测试文件
fn setup_temp_dir(files: &[(&str, &[u8])]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dupfind_test_{}", uuid()));
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

fn uuid() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:08x}_{:04x}", nanos, seq)
}

#[test]
fn test_scan_all_files() {
    let dir = setup_temp_dir(&[
        ("a.txt", b"hello"),
        ("b.txt", b"world"),
        ("c.jpg", b"image"),
    ]);

    let config = ScanConfig {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude_patterns: vec![],
        type_filter: vec![],
    };

    let (files, summary) = scanner::scan(&config).unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(summary.total_files, 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_min_size_filter() {
    let dir = setup_temp_dir(&[("small.txt", b"hi"), ("large.txt", b"hello world")]);

    let config = ScanConfig {
        path: dir.clone(),
        min_size: Some(10),
        extensions: vec![],
        exclude_patterns: vec![],
        type_filter: vec![],
    };

    let (files, _) = scanner::scan(&config).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].path.ends_with("large.txt"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_extension_filter() {
    let dir = setup_temp_dir(&[
        ("photo.jpg", b"image-data"),
        ("notes.txt", b"text-data"),
        ("graphic.png", b"png-data"),
    ]);

    let config = ScanConfig {
        path: dir.clone(),
        min_size: None,
        extensions: vec!["jpg".into(), "png".into()],
        exclude_patterns: vec![],
        type_filter: vec![],
    };

    let (files, _) = scanner::scan(&config).unwrap();
    assert_eq!(files.len(), 2);
    let names: Vec<String> = files
        .iter()
        .map(|f| f.path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.contains(&"photo.jpg".to_string()));
    assert!(names.contains(&"graphic.png".to_string()));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_exclude_filter() {
    let dir = setup_temp_dir(&[
        ("src/main.rs", b"code"),
        ("node_modules/pkg/index.js", b"js"),
        ("target/debug/app.exe", b"exe"),
    ]);

    let config = ScanConfig {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude_patterns: vec!["node_modules".into(), "target".into()],
        type_filter: vec![],
    };

    let (files, _) = scanner::scan(&config).unwrap();

    // CI 调试：打印扫描到的文件路径
    eprintln!("DEBUG test_exclude_filter: found {} files:", files.len());
    for (i, f) in files.iter().enumerate() {
        eprintln!("  [{}] {}", i, f.path.display());
    }

    // 验证排除的路径没有出现
    for f in &files {
        let path_str = f.path.to_string_lossy();
        assert!(
            !path_str.contains("node_modules"),
            "排除路径不应出现: {path_str}"
        );
        assert!(!path_str.contains("target"), "排除路径不应出现: {path_str}");
    }

    // 验证期望的文件存在
    assert!(
        files.iter().any(|f| f.path.ends_with("main.rs")),
        "应包含 src/main.rs"
    );

    let _ = fs::remove_dir_all(&dir);
}
