use crate::token::parse_command_chain;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::AsyncBufReadExt;
use tokio::signal::unix::{signal, SignalKind};

mod exec;
mod prompt;
mod token;
mod shrc;
mod output;
mod history;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static IS_WAITING_FOR_INPUT: AtomicBool = AtomicBool::new(true);

#[cfg(unix)]
async fn sigint_handler() {
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");
    loop {
        sigint.recv().await;
        if IS_WAITING_FOR_INPUT.load(Ordering::SeqCst) {
            print_error!("\n\r");
            prompt::print_prompt();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(unix)]
    tokio::task::spawn(sigint_handler());

    if let Err(e) = shrc::load_shrc().await {
        println_error!("Error loading ~/.shrc: {}", e);
    }

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());

    loop {
        IS_WAITING_FOR_INPUT.store(true, Ordering::SeqCst);
        prompt::print_prompt();

        let mut input = String::new();
        match reader.read_line(&mut input).await {
            Ok(0) => break,
            Ok(_) => {
                let trimmed_input = input.trim();
                if trimmed_input.is_empty() {
                    continue;
                }
                IS_WAITING_FOR_INPUT.store(false, Ordering::SeqCst);
                history::save_history(trimmed_input).await?;
                let tokens = token::tokenize(trimmed_input);
                match parse_command_chain(tokens) {
                    Ok(command_parts) => {
                        if let Err(e) = exec::execute_command_parts(command_parts).await {
                            println_error!("Execution error: {}", e);
                        }
                    }
                    Err(e) => println_error!("Parse error: {}", e),
                }
            }
            Err(e) => {
                println_error!("Error reading input: {}", e);
                break;
            }
        }
    }
    Ok(())
}
