use crate::history;
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub async fn read_command(prompt_with: u16) -> io::Result<String> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    let mut buffer = String::new();
    let mut cursor_pos = 0;
    let mut history_index = 0;

    loop {
        stdout.flush()?;

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    terminal::disable_raw_mode()?;
                    println!("^C");
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "Interrupted"));
                }
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    if buffer.is_empty() {
                        terminal::disable_raw_mode()?;
                        println!("^D");
                        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
                    }
                }
                (KeyCode::Enter, _) => {
                    // Check for line continuation
                    if buffer.trim_end().ends_with('\\') {
                        // Remove backslash and trailing whitespace
                        let trimmed = buffer.trim_end();
                        let len_without_backslash = trimmed.len() - 1;
                        buffer.truncate(len_without_backslash);
                        cursor_pos = buffer.len();

                        // Print newline and continuation prompt
                        stdout.write_all(b"\n\r> ")?;
                        continue;
                    }

                    terminal::disable_raw_mode()?;
                    println!(); // Move to next line
                    return Ok(buffer);
                }
                (KeyCode::Backspace, _) => {
                    if cursor_pos > 0 {
                        buffer.remove(cursor_pos - 1);
                        cursor_pos -= 1;

                        stdout.execute(cursor::MoveLeft(1))?;
                        stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;
                        if cursor_pos < buffer.len() {
                            print!("{}", &buffer[cursor_pos..]);
                            stdout.execute(cursor::MoveLeft((buffer.len() - cursor_pos) as u16))?;
                        }
                    }
                }
                (KeyCode::Left, _) => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        stdout.execute(cursor::MoveLeft(1))?;
                    }
                }
                (KeyCode::Right, _) => {
                    if cursor_pos < buffer.len() {
                        cursor_pos += 1;
                        stdout.execute(cursor::MoveRight(1))?;
                    }
                }
                (KeyCode::Up, _) => {
                    if let Some(history) = history::History::get_by_index(history_index).await {
                        stdout.execute(cursor::MoveToColumn(prompt_with))?;
                        stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;

                        buffer = history;
                        cursor_pos = buffer.len();
                        print!("{}", buffer);
                        history_index += 1;
                        continue;
                    }
                }
                (KeyCode::Down, _) => {
                    stdout.execute(cursor::MoveToColumn(prompt_with))?;
                    stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;
                    if history_index > 0 {
                        history_index -= 1;
                        if let Some(history) = history::History::get_by_index(history_index).await {
                            buffer = history;
                        } else {
                            buffer.clear();
                        }
                        cursor_pos = buffer.len();
                        print!("{}", buffer);
                    } else {
                        buffer.clear();
                        cursor_pos = 0;
                    }
                }
                (KeyCode::Char(c), _) => {
                    if cursor_pos == buffer.len() {
                        buffer.push(c);
                        cursor_pos += 1;
                        print!("{}", c);
                    } else {
                        buffer.insert(cursor_pos, c);
                        cursor_pos += 1;
                        print!("{}", &buffer[cursor_pos - 1..]);
                        stdout.execute(cursor::MoveLeft((buffer.len() - cursor_pos) as u16))?;
                    }
                }
                _ => {}
            }
        }
    }
}
