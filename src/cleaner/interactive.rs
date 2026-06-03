use std::fs;
use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

use crate::error::Result;
use crate::hasher::DuplicateGroup;

/// Run the interactive TUI loop.
///
/// For each duplicate group the user can:
/// - `↑` / `↓` — move cursor
/// - `Space` — toggle whether to **delete** the highlighted file
/// - `Enter` — confirm deletion of all toggled files
/// - `s` — skip this group (keep all files)
/// - `q` — quit immediately
pub fn run(groups: &[DuplicateGroup]) -> Result<()> {
    // Setup terminal: raw mode so we get key-by-key input.
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    let result = run_tui(&mut stdout, groups);

    // Always restore terminal state.
    let _ = terminal::disable_raw_mode();
    let _ = execute!(stdout, cursor::Show);

    result
}

fn run_tui(stdout: &mut impl Write, groups: &[DuplicateGroup]) -> Result<()> {
    execute!(stdout, cursor::Hide)?;

    let total_groups = groups.len();

    for (gi, group) in groups.iter().enumerate() {
        let file_count = group.files.len();
        // `toggled[i] == true` → file will be deleted.
        let mut toggled: Vec<bool> = vec![false; file_count];
        let mut cursor_pos: usize = 0;

        loop {
            // ── Render ──────────────────────────────────────
            render_frame(
                stdout,
                gi,
                total_groups,
                group,
                &toggled,
                cursor_pos,
            )?;

            // ── Input ───────────────────────────────────────
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if cursor_pos > 0 {
                            cursor_pos -= 1;
                        }
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
                        // Delete toggled files.
                        let delete_count = toggled.iter().filter(|&&t| t).count();
                        if delete_count == file_count {
                            // Refuse to delete everything.
                            show_message(
                                stdout,
                                "Cannot delete ALL files — at least one must be kept.",
                            )?;
                            continue;
                        }
                        if delete_count == 0 {
                            show_message(stdout, "No files selected for deletion. Skipping group.")?;
                            break;
                        }
                        execute_deletions(stdout, group, &toggled)?;
                        break;
                    }
                    KeyCode::Char('s') => {
                        // Skip group entirely.
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Quit the whole session.
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c')
                        {
                            // Ctrl-C is handled below only when it's just 'c'.
                        }
                        execute!(
                            stdout,
                            Clear(ClearType::All),
                            cursor::MoveTo(0, 0),
                            Print("Quit by user. Remaining groups were skipped.\r\n"),
                        )?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print("Interactive session finished.\r\n"),
    )?;

    Ok(())
}

/// Draw the current frame.
fn render_frame(
    stdout: &mut impl Write,
    gi: usize,
    total_groups: usize,
    group: &DuplicateGroup,
    toggled: &[bool],
    cursor_pos: usize,
) -> io::Result<()> {
    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    // Header.
    let header = format!(
        "Group {}/{}  |  Hash: {}  |  Size: {} bytes  |  {} files",
        gi + 1,
        total_groups,
        &group.hash[..usize::min(16, group.hash.len())],
        group.size,
        group.files.len(),
    );
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(&header),
        ResetColor,
        Print("\r\n"),
        Print("──────────────────────────────────────────────────────────────────────\r\n"),
    )?;

    // File list.
    for (i, file) in group.files.iter().enumerate() {
        let is_cursor = i == cursor_pos;
        let will_delete = toggled[i];

        // Cursor indicator.
        let cursor_mark = if is_cursor { ">" } else { " " };

        // Deletion marker.
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
            format!(
                "{} {} KEEP   {}",
                cursor_mark,
                mark,
                file.path.display(),
            )
        };

        execute!(stdout, Print(&line), Print("\r\n"))?;
    }

    // Footer help.
    execute!(
        stdout,
        Print("──────────────────────────────────────────────────────────────────────\r\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("[↑/↓] navigate  [Space] toggle delete  [Enter] confirm  [s] skip  [q] quit\r\n"),
        ResetColor,
    )?;

    stdout.flush()
}

/// Perform the actual file deletions indicated by `toggled`.
fn execute_deletions(
    stdout: &mut impl Write,
    group: &DuplicateGroup,
    toggled: &[bool],
) -> io::Result<()> {
    for (i, file) in group.files.iter().enumerate() {
        if toggled[i] {
            match fs::remove_file(&file.path) {
                Ok(()) => {
                    execute!(
                        stdout,
                        Print(format!("  ✓ Deleted: {}\r\n", file.path.display())),
                    )?;
                }
                Err(e) => {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        Print(format!(
                            "  ✗ Failed to delete {}: {}\r\n",
                            file.path.display(),
                            e
                        )),
                        ResetColor,
                    )?;
                }
            }
        }
    }
    execute!(stdout, Print("Press any key to continue...\r\n"))?;
    stdout.flush()?;
    // Drain next key press.
    if let Event::Key(_) = event::read()? {}
    Ok(())
}

/// Show a temporary message and wait for a key press.
fn show_message(stdout: &mut impl Write, msg: &str) -> io::Result<()> {
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(msg),
        Print("\r\nPress any key..."),
    )?;
    stdout.flush()?;
    if let Event::Key(_) = event::read()? {}
    Ok(())
}
