use colored::Colorize;
use std::io::Write;

pub fn get_prompt() -> (String, u16) {
    // 获取当前工作目录
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("?"))
        .display()
        .to_string()
        .blue();
    let green_prompt = "sh>".green();
    // 构建提示符字符串
    (
        format!("{} {} ", current_dir, green_prompt),
        (current_dir.len() + 5) as u16,
    )
}

pub fn print_prompt() -> u16 {
    let (prompt, width) = get_prompt();
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    width
}
