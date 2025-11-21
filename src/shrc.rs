use crate::IS_WAITING_FOR_INPUT;
use crate::token::parse_command_chain;
use crate::{Result, exec, println_error};
use std::env::VarError;
use std::sync::atomic::Ordering;
use tokio::fs;

pub async fn load_shrc() -> Result<()> {
    match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        Ok(home) => {
            let shrc_path = format!("{}/.shrc", home);
            match std::fs::read_to_string(&shrc_path) {
                Ok(contents) => {
                    IS_WAITING_FOR_INPUT.store(false, Ordering::SeqCst);
                    for line in contents.lines() {
                        let trimmed_line = line.trim();
                        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                            continue;
                        }
                        let tokens = crate::token::tokenize(trimmed_line);
                        match parse_command_chain(tokens) {
                            Ok(command_parts) => {
                                if let Err(e) = exec::execute_command_parts(command_parts).await {
                                    println_error!("Error executing {}: {}", shrc_path, e);
                                }
                            }
                            Err(e) => println_error!("Parse error in {}: {}", shrc_path, e),
                        }
                    }
                }
                Err(_) => {
                    let file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&shrc_path)
                        .await?;
                    drop(file);
                }
            }
        }
        Err(_) => return Err(Box::new(VarError::NotPresent)),
    }
    Ok(())
}
