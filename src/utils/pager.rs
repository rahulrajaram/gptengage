//! Built-in alternate-screen pager using crossterm

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

/// RAII guard that restores terminal state on drop (handles panics/early returns)
struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

/// Run an interactive pager over the given content string.
///
/// Enters the alternate screen, displays the content with scroll support,
/// and returns when the user presses q, Esc, or Ctrl-C.
pub fn run_pager(content: &str) -> anyhow::Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut stdout = io::stdout();

    // Enter alternate screen + raw mode
    execute!(stdout, EnterAlternateScreen)?;
    let _guard = RawModeGuard::enter()?;
    execute!(stdout, cursor::Hide)?;

    let mut offset: usize = 0;
    let (mut cols, mut rows) = terminal::size()?;

    loop {
        draw_page(&mut stdout, &lines, offset, cols as usize, rows as usize)?;

        match event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                // Quit
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,

                // Down one line
                KeyCode::Down | KeyCode::Char('j') => {
                    let visible = visible_rows(rows as usize);
                    if offset + visible < lines.len() {
                        offset += 1;
                    }
                }

                // Up one line
                KeyCode::Up | KeyCode::Char('k') => {
                    offset = offset.saturating_sub(1);
                }

                // Page down
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    let visible = visible_rows(rows as usize);
                    offset = offset
                        .saturating_add(visible)
                        .min(lines.len().saturating_sub(visible));
                }

                // Page up
                KeyCode::PageUp => {
                    let visible = visible_rows(rows as usize);
                    offset = offset.saturating_sub(visible);
                }

                // Home / g
                KeyCode::Home | KeyCode::Char('g') => {
                    offset = 0;
                }

                // End / G
                KeyCode::End | KeyCode::Char('G') => {
                    let visible = visible_rows(rows as usize);
                    offset = lines.len().saturating_sub(visible);
                }

                _ => {}
            },
            Event::Resize(new_cols, new_rows) => {
                cols = new_cols;
                rows = new_rows;
                // Clamp offset after resize
                let visible = visible_rows(rows as usize);
                if offset + visible > lines.len() && lines.len() > visible {
                    offset = lines.len() - visible;
                }
            }
            _ => {}
        }
    }

    // Restore terminal
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    // _guard drops here, disabling raw mode

    Ok(())
}

/// Number of content rows (total rows minus 1 for the status bar)
fn visible_rows(total_rows: usize) -> usize {
    total_rows.saturating_sub(1)
}

/// Draw the visible page of content plus a status bar on the last row
fn draw_page(
    stdout: &mut io::Stdout,
    lines: &[&str],
    offset: usize,
    cols: usize,
    rows: usize,
) -> io::Result<()> {
    let visible = visible_rows(rows);
    let end = (offset + visible).min(lines.len());

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All)
    )?;

    // Draw content lines
    for line in lines.iter().take(end).skip(offset) {
        // Truncate lines wider than the terminal (ANSI codes may make this imprecise,
        // but it prevents wrapping artifacts for most cases)
        if line.len() > cols {
            write!(stdout, "{}\r\n", &line[..cols])?;
        } else {
            write!(stdout, "{}\r\n", line)?;
        }
    }

    // Fill remaining rows with blank lines
    for _ in end.saturating_sub(offset)..visible {
        write!(stdout, "\r\n")?;
    }

    // Status bar on the last row (dim text)
    let first = if lines.is_empty() { 0 } else { offset + 1 };
    let status = format!(
        " Lines {}-{} of {}  |  q quit  \u{2191}\u{2193} scroll",
        first,
        end,
        lines.len()
    );
    // Dim attribute
    write!(
        stdout,
        "\x1b[2m{}\x1b[0m",
        if status.len() > cols {
            &status[..cols]
        } else {
            &status
        }
    )?;

    stdout.flush()?;
    Ok(())
}
