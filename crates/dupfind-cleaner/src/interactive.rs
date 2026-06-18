use std::fs;
use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use super::CleanOptions;
use dupfind_core::error::Result;
use dupfind_core::{format_bytes, DuplicateGroup};

/// 交互式 TUI 清理（ratatui 仪表盘）
///
/// 每组重复文件展示给用户，支持：
/// - `↑` / `↓` — 移动光标
/// - `Space` — 标记/取消删除
/// - `Enter` — 确认删除标记的文件
/// - `s` — 跳过本组
/// - `q` — 退出
pub fn run(groups: &[DuplicateGroup], options: &CleanOptions) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui(&mut terminal, groups, options);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_tui(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    groups: &[DuplicateGroup],
    options: &CleanOptions,
) -> Result<()> {
    let total_groups = groups.len();
    let is_dry_run = options.dry_run;

    for (gi, group) in groups.iter().enumerate() {
        let file_count = group.files.len();
        let mut toggled: Vec<bool> = vec![false; file_count];
        let mut cursor_pos: usize = 0;

        loop {
            terminal.draw(|f| {
                render_frame(f, gi, total_groups, group, &toggled, cursor_pos, is_dry_run);
            })?;

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
                            show_popup(terminal, "不能删除所有文件 — 必须保留至少一个。")?;
                            continue;
                        }
                        if delete_count == 0 {
                            break; // 未选择，跳过
                        }
                        execute_deletions(terminal, group, &toggled, options)?;
                        break;
                    }
                    KeyCode::Char('s') => break,
                    KeyCode::Char('q') | KeyCode::Esc => {
                        show_popup(terminal, "用户退出，剩余组已跳过。")?;
                        return Ok(());
                    }
                    _ => {
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            show_popup(terminal, "Ctrl-C 退出。")?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    show_popup(terminal, "交互式清理完成。")?;
    Ok(())
}

/// 渲染单帧
fn render_frame(
    f: &mut ratatui::Frame,
    gi: usize,
    total_groups: usize,
    group: &DuplicateGroup,
    toggled: &[bool],
    cursor_pos: usize,
    is_dry_run: bool,
) {
    // 主布局
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(f.area());

    // 标题区域
    let mode_tag = if is_dry_run { " [DRY-RUN]" } else { "" };
    let header_text = format!(
        "组 {}/{}  |  哈希: {}…  |  大小: {}  |  {} 个文件{}",
        gi + 1,
        total_groups,
        &group.hash[..usize::min(16, group.hash.len())],
        format_bytes(group.size),
        group.files.len(),
        mode_tag,
    );

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" dupfind "))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // 文件列表区域
    let items: Vec<ListItem> = group
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_cursor = i == cursor_pos;
            let will_delete = toggled[i];

            let (prefix, style) = if will_delete {
                ("[✕] DELETE  ", Style::default().fg(Color::Red))
            } else if is_cursor {
                (
                    "[ ] ▶ KEEP  ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("[ ]   KEEP  ", Style::default().fg(Color::DarkGray))
            };

            let path_str = file.path.to_string_lossy().to_string();
            let line = Line::from(vec![Span::styled(prefix, style), Span::raw(path_str)]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" 文件 "))
        .highlight_style(Style::default().bg(Color::DarkGray));
    f.render_widget(list, chunks[1]);

    // 底部帮助
    let help = Paragraph::new("[↑/↓] 导航  [Space] 标记删除  [Enter] 确认  [s] 跳过  [q] 退出")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

/// 执行文件删除
fn execute_deletions(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    group: &DuplicateGroup,
    toggled: &[bool],
    options: &CleanOptions,
) -> io::Result<()> {
    let mut messages: Vec<String> = Vec::new();

    for (i, file) in group.files.iter().enumerate() {
        if toggled[i] {
            if options.dry_run {
                messages.push(format!("  [DRY-RUN] 将删除: {}", file.path.display()));
            } else {
                let result = if options.use_trash {
                    trash::delete(&file.path)
                        .map_err(|e| io::Error::other(format!("回收站操作失败: {e}")))
                } else {
                    fs::remove_file(&file.path)
                };

                match result {
                    Ok(()) => messages.push(format!("  ✓ 已删除: {}", file.path.display())),
                    Err(e) => messages.push(format!("  ✗ 删除失败 {}: {}", file.path.display(), e)),
                }
            }
        }
    }

    messages.push("按任意键继续...".into());
    show_popup(terminal, &messages.join("\n"))?;
    Ok(())
}

/// 显示弹出消息并等待按键
fn show_popup(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    msg: &str,
) -> io::Result<()> {
    terminal.draw(|f| {
        let area = f.area();
        let popup_width = (msg.lines().map(|l| l.len()).max().unwrap_or(40) + 4) as u16;
        let popup_height = (msg.lines().count() + 2) as u16;

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area =
            ratatui::layout::Rect::new(x, y, popup_width.min(area.width), popup_height);

        // 半透明背景遮罩
        let overlay = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(overlay, area);

        let paragraph = Paragraph::new(msg.to_string())
            .block(Block::default().borders(Borders::ALL).title(" 提示 "))
            .style(Style::default().fg(Color::White));
        f.render_widget(paragraph, popup_area);
    })?;

    // 等待按键
    if let Event::Key(_) = event::read()? {}
    Ok(())
}
