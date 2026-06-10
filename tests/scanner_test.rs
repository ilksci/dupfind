use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dupfind::cli::CliArgs;
use dupfind::scanner;

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
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:08x}", nanos)
}

#[test]
fn test_scan_all_files() {
    let dir = setup_temp_dir(&[
        ("a.txt", b"hello"),
        ("b.txt", b"world"),
        ("c.jpg", b"image"),
    ]);

    let args = CliArgs {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude: vec![],
        output: None,
        delete: None,
        dry_run: false,
        use_trash: false,
        table: false,
        hash_algo: dupfind::cli::HashAlgoArg::Blake3,
        config: None,
        verbose: 0,
    };

    let (files, summary) = scanner::scan(&args).unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(summary.total_files, 3);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_min_size_filter() {
    let dir = setup_temp_dir(&[("small.txt", b"hi"), ("large.txt", b"hello world")]);

    let args = CliArgs {
        path: dir.clone(),
        min_size: Some(10),
        extensions: vec![],
        exclude: vec![],
        output: None,
        delete: None,
        dry_run: false,
        use_trash: false,
        table: false,
        hash_algo: dupfind::cli::HashAlgoArg::Blake3,
        config: None,
        verbose: 0,
    };

    let (files, _) = scanner::scan(&args).unwrap();
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

    let args = CliArgs {
        path: dir.clone(),
        min_size: None,
        extensions: vec!["jpg".into(), "png".into()],
        exclude: vec![],
        output: None,
        delete: None,
        dry_run: false,
        use_trash: false,
        table: false,
        hash_algo: dupfind::cli::HashAlgoArg::Blake3,
        config: None,
        verbose: 0,
    };

    let (files, _) = scanner::scan(&args).unwrap();
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

    let args = CliArgs {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude: vec!["node_modules".into(), "target".into()],
        output: None,
        delete: None,
        dry_run: false,
        use_trash: false,
        table: false,
        hash_algo: dupfind::cli::HashAlgoArg::Blake3,
        config: None,
        verbose: 0,
    };

    let (files, _) = scanner::scan(&args).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].path.ends_with("main.rs"));

    let _ = fs::remove_dir_all(&dir);
}
