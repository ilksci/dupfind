use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dupfind::hasher::algorithms::{Blake3Algo, HashAlgorithm, Sha256Algo};

/// 生成指定大小的测试数据
fn make_data(size_kb: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size_kb * 1024);
    for i in 0..(size_kb * 256) {
        data.extend_from_slice(&(i as u32).to_le_bytes());
    }
    data
}

fn bench_sha256(c: &mut Criterion) {
    let algo = Sha256Algo;
    let tiny = make_data(4); // 4 KiB
    let medium = make_data(1024); // 1 MiB
    let large = make_data(10240); // 10 MiB

    let mut group = c.benchmark_group("SHA-256");
    group.throughput(criterion::Throughput::Bytes(tiny.len() as u64));
    group.bench_function("4 KiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&tiny))).unwrap())
    });

    group.throughput(criterion::Throughput::Bytes(medium.len() as u64));
    group.bench_function("1 MiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&medium))).unwrap())
    });

    group.throughput(criterion::Throughput::Bytes(large.len() as u64));
    group.bench_function("10 MiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&large))).unwrap())
    });
    group.finish();
}

fn bench_blake3(c: &mut Criterion) {
    let algo = Blake3Algo;
    let tiny = make_data(4); // 4 KiB
    let medium = make_data(1024); // 1 MiB
    let large = make_data(10240); // 10 MiB

    let mut group = c.benchmark_group("BLAKE3");
    group.throughput(criterion::Throughput::Bytes(tiny.len() as u64));
    group.bench_function("4 KiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&tiny))).unwrap())
    });

    group.throughput(criterion::Throughput::Bytes(medium.len() as u64));
    group.bench_function("1 MiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&medium))).unwrap())
    });

    group.throughput(criterion::Throughput::Bytes(large.len() as u64));
    group.bench_function("10 MiB", |b| {
        b.iter(|| algo.hash(&mut Cursor::new(black_box(&large))).unwrap())
    });
    group.finish();
}

criterion_group!(benches, bench_sha256, bench_blake3);
criterion_main!(benches);
