pub mod cleaner;
pub mod cli;
pub mod config;
pub mod error;
pub mod hasher;
pub mod reporter;
pub mod scanner;
pub mod table;

use hasher::algorithms::{Blake3Algo, HashAlgorithm, Sha256Algo};

/// 顶层入口
pub fn run() -> error::Result<()> {
    let args = cli::parse_args();

    // 初始化日志（-v 或 RUST_LOG 环境变量控制）
    init_logger(args.verbose);

    // 加载配置文件（v3 将读取更多字段用于参数合并）
    let _cfg = config::Config::load(args.config.as_ref());

    // ── 阶段 1: 扫描 ──────────────────────────────
    let (file_infos, summary) = scanner::scan(&args)?;

    if file_infos.is_empty() {
        println!("没有找到符合条件的文件。");
        return Ok(());
    }

    println!("扫描完成: {} 个文件", summary.total_files,);
    if summary.symlinks > 0 {
        println!("  （跳过 {} 个符号链接）", summary.symlinks);
    }

    // ── 阶段 2: 哈希查重 ──────────────────────────
    let algo: Box<dyn HashAlgorithm> = match args.hash_algo {
        cli::HashAlgoArg::Blake3 => Box::new(Blake3Algo),
        cli::HashAlgoArg::Sha256 => Box::new(Sha256Algo),
    };

    println!("使用 {} 算法计算哈希...", algo.name());
    let duplicates = hasher::find_duplicates(file_infos, algo.as_ref())?;

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
        cleaner_mod_bytes(wasted_bytes),
    );

    // ── 阶段 3: 终端表格输出（可选）───────────────
    if args.table {
        table::print_table(&duplicates);
    }

    // ── 阶段 4: 导出报告（可选）───────────────────
    if let Some(ref output_path) = args.output {
        reporter::export(&duplicates, output_path)?;
        println!("报告已导出到: {}", output_path.display());
    }

    // ── 阶段 5: 清理重复文件（可选）───────────────
    if let Some(ref strategy_arg) = args.delete {
        let strategy: cleaner::KeepStrategy = strategy_arg.clone().into();
        let options = cleaner::CleanOptions {
            dry_run: args.dry_run,
            use_trash: args.use_trash,
        };
        cleaner::clean(&duplicates, &strategy, &options)?;
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

fn cleaner_mod_bytes(bytes: u64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("TB", 1_099_511_627_776.0),
        ("GB", 1_073_741_824.0),
        ("MB", 1_048_576.0),
        ("KB", 1_024.0),
        ("B", 1.0),
    ];
    for (unit, div) in UNITS {
        let val = bytes as f64 / div;
        if val >= 1.0 || *unit == "B" {
            return format!("{:.1} {}", val, unit);
        }
    }
    format!("{bytes} B")
}
