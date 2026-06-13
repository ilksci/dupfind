use std::fs;
use std::io::Write;
use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dupfind::scanner::{self, ScanConfig};

/// 创建临时测试目录结构
fn setup_bench_dir(file_count: usize) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dupfind_bench_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    for i in 0..file_count {
        let subdir = dir.join(format!("sub_{}", i % 10));
        fs::create_dir_all(&subdir).unwrap();
        let path = subdir.join(format!("file_{}.txt", i));
        let mut f = fs::File::create(&path).unwrap();
        let content = format!("benchmark file number {}\n", i).repeat((i % 50) + 1);
        f.write_all(content.as_bytes()).unwrap();
    }

    dir
}

fn bench_scan_small(c: &mut Criterion) {
    let dir = setup_bench_dir(100);
    let config = ScanConfig {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude_patterns: vec![],
        type_filter: vec![],
    };

    c.bench_function("scan 100 files", |b| {
        b.iter(|| {
            let _ = scanner::scan(black_box(&config)).unwrap();
        })
    });

    let _ = fs::remove_dir_all(&dir);
}

fn bench_scan_medium(c: &mut Criterion) {
    let dir = setup_bench_dir(500);
    let config = ScanConfig {
        path: dir.clone(),
        min_size: None,
        extensions: vec![],
        exclude_patterns: vec![],
        type_filter: vec![],
    };

    c.bench_function("scan 500 files", |b| {
        b.iter(|| {
            let _ = scanner::scan(black_box(&config)).unwrap();
        })
    });

    let _ = fs::remove_dir_all(&dir);
}

criterion_group!(benches, bench_scan_small, bench_scan_medium);
criterion_main!(benches);
