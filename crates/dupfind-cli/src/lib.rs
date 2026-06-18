pub mod cli;
pub mod config;
pub mod serve;
pub mod table;

use dupfind_core::format_bytes;
use dupfind_core::HashAlgorithm;
use dupfind_hasher::algorithms::{Blake3Algo, Sha256Algo};

/// 顶层入口
pub fn run() -> dupfind_core::error::Result<()> {
    let args = cli::parse_args();

    // 初始化日志（-v 或 RUST_LOG 环境变量控制）
    init_logger(args.verbose);

    // 加载配置文件（v3 将读取更多字段用于参数合并）
    let _cfg = config::Config::load(args.config.as_ref());

    // ── 阶段 1: 扫描 ──────────────────────────────
    let scan_config = dupfind_scanner::ScanConfig {
        path: args.path.clone(),
        min_size: args.min_size,
        extensions: args.extensions.clone(),
        exclude_patterns: args.exclude.clone(),
        type_filter: args.type_filter.clone(),
    };
    let (file_infos, summary) = dupfind_scanner::scan(&scan_config)?;

    if file_infos.is_empty() {
        println!("没有找到符合条件的文件。");
        return Ok(());
    }

    println!("扫描完成: {} 个文件", summary.total_files);
    if summary.symlinks > 0 {
        println!("  （跳过 {} 个符号链接）", summary.symlinks);
    }
    // 显示文件类型分布
    if !summary.type_distribution.is_empty() {
        println!("  文件类型分布:");
        let mut dist: Vec<_> = summary.type_distribution.iter().collect();
        dist.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
        for (ftype, count) in dist.iter().take(8) {
            println!("    {:30} {}", ftype, count);
        }
        if dist.len() > 8 {
            println!("    ... 共 {} 种类型", dist.len());
        }
    }

    // ── 阶段 2: 查重 ──────────────────────────────
    if args.find_similar {
        // 相似文件检测模式 (Phase 3d)
        return run_similar_mode(&args, &file_infos);
    }

    let algo: Box<dyn HashAlgorithm> = match args.hash_algo {
        cli::HashAlgoArg::Blake3 => Box::new(Blake3Algo),
        cli::HashAlgoArg::Sha256 => Box::new(Sha256Algo),
    };

    println!("使用 {} 算法计算哈希...", algo.name());

    let duplicates = if args.use_async {
        // 异步哈希路径 (Phase 3c)
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            dupfind_core::DupfindError::Other(format!("无法创建 tokio runtime: {e}"))
        })?;
        rt.block_on(async {
            use indicatif::{ProgressBar, ProgressStyle};
            use std::io;

            let total = file_infos.len() as u64;
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} 异步哈希计算 [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}, 预计 {eta})",
                    )
                    .unwrap()
                    .progress_chars("━╸ "),
            );

            let mut hashed = Vec::with_capacity(file_infos.len());
            for mut f in file_infos {
                pb.inc(1);
                match tokio::fs::read(&f.path).await {
                    Ok(data) => {
                        let mut cursor = io::Cursor::new(&data);
                        f.hash = algo.hash(&mut cursor).ok();
                    }
                    Err(e) => {
                        log::debug!("异步读取失败 {}: {}", f.path.display(), e);
                    }
                }
                hashed.push(f);
            }
            pb.finish_and_clear();

            // 分组
            use std::collections::HashMap;
            let mut hash_buckets: HashMap<String, Vec<dupfind_core::FileInfo>> = HashMap::new();
            for f in hashed {
                if let Some(ref hash) = f.hash {
                    hash_buckets.entry(hash.clone()).or_default().push(f);
                }
            }
            let groups: Vec<dupfind_core::DuplicateGroup> = hash_buckets
                .into_iter()
                .filter(|(_, b)| b.len() >= 2)
                .map(|(hash, bucket)| {
                    let size = bucket.first().map(|f| f.size).unwrap_or(0);
                    dupfind_core::DuplicateGroup { hash, size, files: bucket }
                })
                .collect();
            Ok::<_, dupfind_core::DupfindError>(groups)
        })?
    } else {
        // 同步哈希路径（默认）
        dupfind_hasher::find_duplicates(file_infos, algo.as_ref())?
    };

    if duplicates.is_empty() {
        println!("未发现重复文件。");
        return Ok(());
    }

    let total_dup_count: usize = duplicates.iter().map(|g| g.files.len()).sum();
    let wasted_bytes: u64 = duplicates
        .iter()
        .map(|g| g.size * (g.files.len() - 1) as u64)
        .sum();

    println!(
        "发现 {} 个重复组，共 {} 个重复文件，可释放约 {}。",
        duplicates.len(),
        total_dup_count,
        format_bytes(wasted_bytes),
    );

    // ── 阶段 3: 终端表格输出（可选）───────────────
    if args.table {
        table::print_table(&duplicates);
    }

    // ── 阶段 4: 导出报告（可选）───────────────────
    if let Some(ref output_path) = args.output {
        dupfind_reporter::export(&duplicates, output_path)?;
        println!("报告已导出到: {}", output_path.display());
    }

    // ── 阶段 5: Web 服务器（可选）──────────────────
    if let Some(port) = args.serve {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            dupfind_core::DupfindError::Other(format!("无法创建 tokio runtime: {e}"))
        })?;
        rt.block_on(async {
            serve::start_server(duplicates, total_dup_count, wasted_bytes, port).await;
        });
        return Ok(());
    }

    // ── 阶段 6: 清理重复文件（可选）───────────────
    if let Some(ref strategy_arg) = args.delete {
        let strategy: dupfind_cleaner::KeepStrategy = strategy_arg.clone().into();
        let options = dupfind_cleaner::CleanOptions {
            dry_run: args.dry_run,
            use_trash: args.use_trash,
        };
        dupfind_cleaner::clean(&duplicates, &strategy, &options)?;
    }

    Ok(())
}

/// 相似文件模式
fn run_similar_mode(
    args: &cli::CliArgs,
    file_infos: &[dupfind_core::FileInfo],
) -> dupfind_core::error::Result<()> {
    use dupfind_core::format_bytes;

    println!("使用相似文件检测模式（阈值: {}%）...", args.threshold);

    // 分离图片和文本文件
    let images: Vec<_> = file_infos
        .iter()
        .filter(|f| {
            f.detected_type
                .as_ref()
                .map(|t| t.starts_with("image/"))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let texts: Vec<_> = file_infos
        .iter()
        .filter(|f| {
            f.detected_type
                .as_ref()
                .map(|t| t.starts_with("text/"))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    // 图片感知哈希
    if !images.is_empty() {
        println!("  分析 {} 张图片...", images.len());
        let img_groups = dupfind_hasher::similar::find_similar_images(&images, args.threshold);
        for g in &img_groups {
            println!("  相似图片组 ({} 文件, {}):", g.files.len(), g.reason);
            for f in &g.files {
                println!("    {}", f.path.display());
            }
        }
        let wasted: u64 = img_groups
            .iter()
            .map(|g| g.files.iter().map(|f| f.size).sum::<u64>() - g.files[0].size)
            .sum();
        println!(
            "  图片: {} 组相似, 可释放约 {}",
            img_groups.len(),
            format_bytes(wasted),
        );
    }

    // 文本相似度
    if !texts.is_empty() {
        println!("  分析 {} 个文本文件...", texts.len());
        let text_groups = dupfind_hasher::similar::find_similar_text(&texts, args.threshold);
        for g in &text_groups {
            println!("  相似文本组 ({} 文件, {}):", g.files.len(), g.reason);
            for f in &g.files {
                println!("    {}", f.path.display());
            }
        }
        let wasted: u64 = text_groups
            .iter()
            .map(|g| g.files.iter().map(|f| f.size).sum::<u64>() - g.files[0].size)
            .sum();
        println!(
            "  文本: {} 组相似, 可释放约 {}",
            text_groups.len(),
            format_bytes(wasted),
        );
    }

    Ok(())
}

/// 初始化日志系统
fn init_logger(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };

    // 尊重 RUST_LOG 环境变量
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", level);
    }
    let _ = env_logger::try_init();
}

// 当作为 library 使用时，重新导出 run
pub use run as main_entry;
