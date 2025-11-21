use crate::Result;
use tokio::io::AsyncWriteExt;
pub async fn save_history(command: &str) -> Result<()> {
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
        Err(e) => {
            Err(Box::new(e))
        }
    }
}