use crate::token::{CommandPart, ExecutionSource, PipeEndpoint};
use crate::{println_error, Result};
use std::env;
use std::fs::File;
use std::process::Command;

pub(crate) async fn execute_command_parts(parts: Vec<CommandPart>) -> Result<()> {
    if parts.is_empty() {
        return Ok(());
    }

    // 检查内置命令 (只能是第一个命令)
    let CommandPart::Execute { name, args, .. } = &parts[0];
    match name.as_str() {
        "exit" => std::process::exit(0),
        "cd" => {
            let home_path = env::var("HOME").unwrap_or_else(|_| "/".to_string());
            let path = args.get(0).unwrap_or(&home_path);
            let new_dir = std::path::Path::new(path);
            if let Err(e) = env::set_current_dir(new_dir) {
                println_error!("cd error: {}", e);
            }
            return Ok(());
        }
        _ => {}
    }

    let mut previous_stdout_handle: Option<std::process::ChildStdout> = None;

    // 遍历执行命令链
    for part in parts.into_iter() {
        let CommandPart::Execute {
            name,
            args,
            stdin,
            stdout,
        } = part;

        let mut command = Command::new(&name);
        command.args(&args);

        // --- 设置 STDIN ---
        match stdin {
            ExecutionSource::Inherit => {
                command.stdin(std::process::Stdio::inherit());
            }
            ExecutionSource::Pipe(PipeEndpoint::Read) => {
                if let Some(handle) = previous_stdout_handle.take() {
                    command.stdin(handle);
                } else {
                    // 错误情况：管道没有上游，应继承 stdin
                    command.stdin(std::process::Stdio::inherit());
                }
            }
            ExecutionSource::File(path) => {
                let file = File::open(path)?;
                command.stdin(std::process::Stdio::from(file));
            }
            _ => {}
        }

        // --- 设置 STDOUT ---
        let is_piped = match stdout {
            ExecutionSource::Inherit => {
                command.stdout(std::process::Stdio::inherit());
                false
            }
            ExecutionSource::Pipe(PipeEndpoint::Write) => {
                command.stdout(std::process::Stdio::piped());
                true
            }
            ExecutionSource::File(path) => {
                let file = if path.starts_with(">>") {
                    // 追加重定向
                    File::options().append(true).create(true).open(&path[2..])?
                } else {
                    // 覆盖重定向
                    File::create(path)?
                };
                command.stdout(std::process::Stdio::from(file));
                false
            }
            _ => false,
        };

        // --- 执行 ---
        let mut child = command.spawn()?;

        // --- 存储管道句柄 或 等待完成 ---
        if is_piped {
            // 如果是管道输出，保存输出句柄给下一个命令
            previous_stdout_handle = child.stdout.take();
        } else if previous_stdout_handle.is_none() {
            // 如果不是管道输出，且没有未连接的管道 (即是链条的终点或单个命令)
            child.wait()?;
        }
        // 否则，如果是链条终点，但前面还有未等待的命令，我们只等待链条的最后一个
    }

    Ok(())
}
