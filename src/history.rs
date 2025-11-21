use crate::Result;
use lazy_static::lazy_static;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

lazy_static! {
    static ref HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub struct History;

impl History {
    pub async fn load() -> Result<()> {
        let mut history = HISTORY.lock().await;
        match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(home) => {
                let history_path = format!("{}/.sh_history", home);
                if let Ok(file) = tokio::fs::File::open(&history_path).await {
                    let reader = tokio::io::BufReader::new(file);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        history.push(line);
                    }
                }
            }
            Err(_) => {}
        }
        Ok(())
    }
    pub async fn save(command: &str) -> Result<()> {
        let mut history = HISTORY.lock().await;
        history.push(command.to_string());
        drop(history);
        match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            Ok(home) => {
                let history_path = format!("{}/.sh_history", home);
                let mut file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&history_path)
                    .await?;
                file.write_all(format!("{}\n", command).as_bytes()).await?;
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    /// 倒序获取历史命令，0 为最新的命令
    pub async fn get_by_index(index: usize) -> Option<String> {
        let history = HISTORY.lock().await;
        if index < history.len() {
            Some(history[history.len() - 1 - index].clone())
        } else {
            None
        }
    }
}
