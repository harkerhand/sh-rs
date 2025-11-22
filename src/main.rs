use crate::interrupt::sigint_handler;
use crate::token::parse_command_chain;
use std::sync::atomic::{AtomicBool, Ordering};

mod exec;
mod history;
mod input;
mod interrupt;
mod output;
mod prompt;
mod shrc;
mod token;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static IS_WAITING_FOR_INPUT: AtomicBool = AtomicBool::new(true);

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(unix)]
    tokio::task::spawn(sigint_handler());

    if let Err(e) = shrc::load_shrc().await {
        println_error!("Error loading ~/.shrc: {}", e);
    }
    if let Err(e) = history::History::load().await {
        println_error!("Error loading history: {}", e);
    }
    loop {
        IS_WAITING_FOR_INPUT.store(true, Ordering::SeqCst);
        let width = prompt::print_prompt();
        match input::read_command(width).await {
            Ok(input) => {
                let trimmed_input = input.trim();
                if trimmed_input.is_empty() {
                    continue;
                }
                IS_WAITING_FOR_INPUT.store(false, Ordering::SeqCst);
                history::History::save(trimmed_input).await?;

                match parse_command_chain(token::tokenize(trimmed_input)) {
                    Ok(command_parts) => {
                        if let Err(e) = exec::execute_command_parts(command_parts).await {
                            println_error!("Execution error: {}", e);
                        }
                    }
                    Err(e) => println_error!("Parse error: {}", e),
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                } else if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    break;
                }
                println_error!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}
