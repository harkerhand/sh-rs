use std::io::Write;

pub fn get_prompt() -> String {
    // 获取当前工作目录
    let current_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("?"))
        .display()
        .to_string();

    // 构建提示符字符串
    format!("{} sh> ", current_dir)
}

pub fn print_prompt() {
    print!("{}", get_prompt());
    std::io::stdout().flush().unwrap();
}
