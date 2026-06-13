pub mod interactive;

use std::fs;
use std::time::SystemTime;

use dupfind_core::error::{DupfindError, Result};
use dupfind_core::{format_bytes, DuplicateGroup, FileInfo};

/// 文件保留策略
#[derive(Debug, Clone)]
pub enum KeepStrategy {
    /// 保留修改时间最新的文件
    Newest,
    /// 保留修改时间最早的文件
    Oldest,
    /// 保留路径最短的文件
    Shortest,
    /// 保留体积最大的文件
    Largest,
    /// 保留体积最小的文件
    Smallest,
    /// 交互式逐组选择（TUI）
    Interactive,
}

/// 清理配置
pub struct CleanOptions {
    /// 是否为预览模式（dry-run），true 则只打印不删除
    pub dry_run: bool,
    /// 是否移动到回收站
    pub use_trash: bool,
}

/// 执行清理
pub fn clean(
    groups: &[DuplicateGroup],
    strategy: &KeepStrategy,
    options: &CleanOptions,
) -> Result<()> {
    if groups.is_empty() {
        println!("没有重复文件需要清理。");
        return Ok(());
    }

    if options.dry_run {
        println!("═══ DRY-RUN 模式 — 以下是将要删除的文件 ═══\n");
    }

    match strategy {
        KeepStrategy::Interactive => interactive::run(groups, options),
        other => clean_auto(groups, other, options),
    }
}

/// 自动清理：按策略保留一个，删除其余
fn clean_auto(
    groups: &[DuplicateGroup],
    strategy: &KeepStrategy,
    options: &CleanOptions,
) -> Result<()> {
    let mut total_deleted: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut total_errors: usize = 0;

    for group in groups {
        let keeper_idx = choose_keeper(&group.files, strategy);
        let to_delete: Vec<_> = group
            .files
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != keeper_idx)
            .collect();

        for (_, file) in &to_delete {
            if options.dry_run {
                println!("  [DRY-RUN] 将删除: {}", file.path.display());
                total_deleted += 1;
                total_bytes += file.size;
                continue;
            }

            let result = if options.use_trash {
                delete_via_trash(&file.path)
            } else {
                fs::remove_file(&file.path).map_err(|e| e.into())
            };

            match result {
                Ok(()) => {
                    println!("  已删除: {}", file.path.display());
                    total_deleted += 1;
                    total_bytes += file.size;
                }
                Err(e) => {
                    log::error!("删除失败 {}: {}", file.path.display(), e);
                    total_errors += 1;
                }
            }
        }
    }

    if options.dry_run {
        println!(
            "\n═══ 预览完毕: 将删除 {total_deleted} 个文件，释放 {freed} ═══",
            total_deleted = total_deleted,
            freed = format_bytes(total_bytes),
        );
    } else {
        println!(
            "清理完成。删除了 {total_deleted} 个文件，释放了 {freed}。{errors}",
            total_deleted = total_deleted,
            freed = format_bytes(total_bytes),
            errors = if total_errors > 0 {
                format!("（{total_errors} 个文件删除失败）")
            } else {
                String::new()
            },
        );
    }
    Ok(())
}

/// 通过系统回收站删除文件
fn delete_via_trash(path: &std::path::Path) -> Result<()> {
    trash::delete(path)
        .map_err(|e| DupfindError::Trash(format!("无法将 {} 移入回收站: {}", path.display(), e)))
}

/// 根据策略选择保留的文件索引
fn choose_keeper(files: &[FileInfo], strategy: &KeepStrategy) -> usize {
    match strategy {
        KeepStrategy::Newest => files
            .iter()
            .enumerate()
            .max_by_key(|(_, f)| f.modified.unwrap_or(SystemTime::UNIX_EPOCH))
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Oldest => files
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| f.modified.unwrap_or(SystemTime::UNIX_EPOCH))
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Shortest => files
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| f.path.to_string_lossy().len())
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Largest => files
            .iter()
            .enumerate()
            .max_by_key(|(_, f)| f.size)
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Smallest => files
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| f.size)
            .map(|(i, _)| i)
            .unwrap_or(0),

        KeepStrategy::Interactive => 0, // 不会走到这里
    }
}
