//! Paste diagnostic — run with:
//!   cargo run --example paste_diag
//!
//! Enables raw mode + bracketed paste, then prints every crossterm event.
//! Paste some multi-line text and see what events arrive.
//! Press 'q' to quit.

use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Also manually write the escape code to be sure
    write!(stdout, "\x1b[?2004h")?;
    stdout.flush()?;

    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;

    // Print instructions
    execute!(
        stdout,
        crossterm::cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    write!(stdout, "=== Paste Diagnostic ===\r\n")?;
    write!(
        stdout,
        "Paste some multi-line text and watch what events appear.\r\n"
    )?;
    write!(stdout, "Press 'q' to quit.\r\n")?;
    write!(stdout, "---\r\n")?;
    stdout.flush()?;

    let mut line = 4u16;

    loop {
        if event::poll(Duration::from_millis(100))? {
            let evt = event::read()?;

            execute!(stdout, crossterm::cursor::MoveTo(0, line))?;

            match &evt {
                Event::Paste(text) => {
                    let lines = text.lines().count();
                    write!(
                        stdout,
                        "*** PASTE EVENT: {} chars, {} lines, first 60: {:?}\r\n",
                        text.len(),
                        lines,
                        &text[..text.len().min(60)]
                    )?;
                }
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        write!(
                            stdout,
                            "KEY: code={:?}  mods={:?}  kind={:?}\r\n",
                            key.code, key.modifiers, key.kind
                        )?;

                        if key.code == KeyCode::Char('q') {
                            break;
                        }
                    }
                }
                other => {
                    write!(stdout, "OTHER: {:?}\r\n", other)?;
                }
            }
            stdout.flush()?;
            line += 1;

            if line > 50 {
                line = 4;
                execute!(
                    stdout,
                    crossterm::cursor::MoveTo(0, 4),
                    crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
                )?;
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen, DisableBracketedPaste)?;
    disable_raw_mode()?;

    println!("Done.");
    Ok(())
}
