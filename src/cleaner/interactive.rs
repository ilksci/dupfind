use std::fs;
use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use super::CleanOptions;
use crate::error::Result;
use crate::hasher::DuplicateGroup;

/// 交互式 TUI 清理循环
///
/// 每组重复文件展示给用户，支持：
/// - `↑` / `↓` — 移动光标
/// - `Space` — 标记/取消删除
/// - `Enter` — 确认删除标记的文件
/// - `s` — 跳过本组
/// - `q` — 退出
pub fn run(groups: &[DuplicateGroup], options: &CleanOptions) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    let result = run_tui(&mut stdout, groups, options);

    let _ = terminal::disable_raw_mode();
    let _ = execute!(stdout, cursor::Show);

    result
}

fn run_tui(
    stdout: &mut impl Write,
    groups: &[DuplicateGroup],
    options: &CleanOptions,
) -> Result<()> {
    execute!(stdout, cursor::Hide)?;

    let total_groups = groups.len();
    let is_dry_run = options.dry_run;

    for (gi, group) in groups.iter().enumerate() {
        let file_count = group.files.len();
        let mut toggled: Vec<bool> = vec![false; file_count];
        let mut cursor_pos: usize = 0;

        loop {
            render_frame(
                stdout,
                gi,
                total_groups,
                group,
                &toggled,
                cursor_pos,
                is_dry_run,
            )?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        cursor_pos = cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cursor_pos + 1 < file_count {
                            cursor_pos += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        toggled[cursor_pos] = !toggled[cursor_pos];
                    }
                    KeyCode::Enter => {
                        let delete_count = toggled.iter().filter(|&&t| t).count();
                        if delete_count == file_count {
                            show_message(stdout, "不能删除所有文件 — 必须保留至少一个。")?;
                            continue;
                        }
                        if delete_count == 0 {
                            show_message(stdout, "未选择任何文件，跳过本组。")?;
                            break;
                        }
                        execute_deletions(stdout, group, &toggled, options)?;
                        break;
                    }
                    KeyCode::Char('s') => break,
                    KeyCode::Char('q') | KeyCode::Esc => {
                        execute!(
                            stdout,
                            Clear(ClearType::All),
                            cursor::MoveTo(0, 0),
                            Print("用户退出，剩余组已跳过。\r\n"),
                        )?;
                        return Ok(());
                    }
                    _ => {
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            execute!(
                                stdout,
                                Clear(ClearType::All),
                                cursor::MoveTo(0, 0),
                                Print("Ctrl-C 退出。\r\n"),
                            )?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print("交互式清理完成。\r\n"),
    )?;

    Ok(())
}

fn render_frame(
    stdout: &mut impl Write,
    gi: usize,
    total_groups: usize,
    group: &DuplicateGroup,
    toggled: &[bool],
    cursor_pos: usize,
    is_dry_run: bool,
) -> io::Result<()> {
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    // 标题行
    let mode_tag = if is_dry_run { " [DRY-RUN]" } else { "" };
    let header = format!(
        "组 {}/{}  |  哈希: {}…  |  大小: {} 字节  |  {} 个文件{}",
        gi + 1,
        total_groups,
        &group.hash[..usize::min(16, group.hash.len())],
        group.size,
        group.files.len(),
        mode_tag,
    );
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(&header),
        ResetColor,
        Print("\r\n"),
        Print("──────────────────────────────────────────────────────────────────────\r\n"),
    )?;

    // 文件列表
    for (i, file) in group.files.iter().enumerate() {
        let is_cursor = i == cursor_pos;
        let will_delete = toggled[i];

        let cursor_mark = if is_cursor { ">" } else { " " };
        let mark = if will_delete { "[✕]" } else { "[ ]" };

        let line = if will_delete {
            format!(
                "{} {} \x1b[31mDELETE\x1b[0m  {}",
                cursor_mark,
                mark,
                file.path.display(),
            )
        } else if is_cursor {
            format!(
                "{} {} \x1b[1mKEEP\x1b[0m   {}",
                cursor_mark,
                mark,
                file.path.display(),
            )
        } else {
            format!("{} {} KEEP   {}", cursor_mark, mark, file.path.display())
        };

        execute!(stdout, Print(&line), Print("\r\n"))?;
    }

    // 底部帮助
    execute!(
        stdout,
        Print("──────────────────────────────────────────────────────────────────────\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("[↑/↓] 导航  [Space] 标记删除  [Enter] 确认  [s] 跳过  [q] 退出\r\n"),
        ResetColor,
    )?;

    stdout.flush()
}

fn execute_deletions(
    stdout: &mut impl Write,
    group: &DuplicateGroup,
    toggled: &[bool],
    options: &CleanOptions,
) -> io::Result<()> {
    for (i, file) in group.files.iter().enumerate() {
        if toggled[i] {
            if options.dry_run {
                execute!(
                    stdout,
                    Print(format!("  [DRY-RUN] 将删除: {}\r\n", file.path.display())),
                )?;
            } else {
                let result = if options.use_trash {
                    trash::delete(&file.path)
                        .map_err(|e| io::Error::other(format!("回收站操作失败: {e}")))
                } else {
                    fs::remove_file(&file.path)
                };

                match result {
                    Ok(()) => {
                        execute!(
                            stdout,
                            Print(format!("  ✓ 已删除: {}\r\n", file.path.display())),
                        )?;
                    }
                    Err(e) => {
                        execute!(
                            stdout,
                            SetForegroundColor(Color::Red),
                            Print(format!("  ✗ 删除失败 {}: {}\r\n", file.path.display(), e)),
                            ResetColor,
                        )?;
                    }
                }
            }
        }
    }
    execute!(stdout, Print("按任意键继续...\r\n"))?;
    stdout.flush()?;
    if let Event::Key(_) = event::read()? {}
    Ok(())
}

fn show_message(stdout: &mut impl Write, msg: &str) -> io::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(msg),
        Print("\r\n按任意键..."),
    )?;
    stdout.flush()?;
    if let Event::Key(_) = event::read()? {}
    Ok(())
}
