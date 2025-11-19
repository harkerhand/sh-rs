use crate::Result;

// 表示一个最小的词法单元
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Word(String),
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
}

// 表示一个执行单元的抽象语法树 (AST) 节点
#[derive(Debug)]
pub enum CommandPart {
    Execute {
        name: String,
        args: Vec<String>,
        stdin: ExecutionSource,
        stdout: ExecutionSource,
    },
}

#[derive(Debug, PartialEq)]
pub enum ExecutionSource {
    Inherit,
    Pipe(PipeEndpoint),
    File(String),
}

#[derive(Debug, PartialEq)]
pub enum PipeEndpoint {
    Read,
    Write,
}
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current = String::new();
    let mut in_quotes = false;

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                // 注意：在最终的 Word 中去除引号
                if !in_quotes && !current.is_empty() {
                    tokens.push(Token::Word(current.to_string()));
                    current.clear();
                }
            }
            // 遇到非引号内的空格，作为分隔符
            ' ' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.to_string()));
                    current.clear();
                }
            }
            // 遇到操作符，作为分隔符
            '|' | '<' | '>' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(Token::Word(current.to_string()));
                    current.clear();
                }

                // 识别多字符操作符
                match c {
                    '|' => tokens.push(Token::Pipe),
                    '<' => tokens.push(Token::RedirectIn),
                    '>' => {
                        if chars.peek() == Some(&'>') {
                            chars.next(); // 消耗第二个 '>'
                            tokens.push(Token::RedirectAppend);
                        } else {
                            tokens.push(Token::RedirectOut);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    // 处理循环结束时剩余的 current
    if !current.is_empty() {
        tokens.push(Token::Word(current.to_string()));
    }

    // 最终清理：去除 Word token 周围的引号（如果存在）
    tokens
        .into_iter()
        .map(|token| {
            if let Token::Word(s) = token {
                Token::Word(s.trim_matches('"').to_string())
            } else {
                token
            }
        })
        .collect()
}
pub fn parse_command_chain(tokens: Vec<Token>) -> Result<Vec<CommandPart>> {
    let mut parts = Vec::new();
    let mut current_command: Vec<String> = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    // 状态机：跟踪下一个操作符需要什么
    let mut pending_stdin = ExecutionSource::Inherit;

    // 初始输出假设是下一个命令的输入（管道），或继承终端
    let mut pending_stdout = ExecutionSource::Inherit;

    while let Some(token) = iter.next() {
        match token {
            Token::Word(word) => {
                current_command.push(word);
            }
            op @ (Token::Pipe | Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend) => {
                // 1. 检查是否有前一个命令需要封装
                if current_command.is_empty() {
                    return Err(
                        format!("Parse error: Command expected before operator {:?}", op).into(),
                    );
                }

                // 2. 检查操作符
                match op {
                    Token::Pipe => {
                        // 结束当前命令：stdout 设置为 Pipe Write
                        pending_stdout = ExecutionSource::Pipe(PipeEndpoint::Write);

                        // 封装当前命令并推入
                        parts.push(CommandPart::Execute {
                            name: current_command[0].clone(),
                            args: current_command.drain(1..).collect(),
                            stdin: pending_stdin,
                            stdout: pending_stdout,
                        });
                        current_command.clear();

                        // 为下一个命令准备：stdin 设置为 Pipe Read
                        pending_stdin = ExecutionSource::Pipe(PipeEndpoint::Read);
                        pending_stdout = ExecutionSource::Inherit; // 重置下一个命令的默认输出
                    }
                    // 重定向操作：从迭代器中获取文件名
                    _ => {
                        if let Some(Token::Word(filename)) = iter.next() {
                            match op {
                                Token::RedirectIn => {
                                    pending_stdin = ExecutionSource::File(filename)
                                }
                                Token::RedirectOut => {
                                    pending_stdout = ExecutionSource::File(filename)
                                }
                                Token::RedirectAppend => {
                                    // 使用一个特殊的命名约定或结构来区分 Append
                                    pending_stdout =
                                        ExecutionSource::File(format!(">>{}", filename));
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            return Err(format!("Parse error: Redirection operator {:?} must be followed by a filename.", op).into());
                        }
                    }
                }
            }
        }
    }

    // 处理最后一个命令
    if !current_command.is_empty() {
        parts.push(CommandPart::Execute {
            name: current_command[0].clone(),
            args: current_command.drain(1..).collect(),
            stdin: pending_stdin,
            stdout: pending_stdout,
        });
    }

    Ok(parts)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_redir() {
        let input = "echo 123 >> a.txt ";
        let tokens = tokenize(input);
        let expected_tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("123".to_string()),
            Token::RedirectAppend,
            Token::Word("a.txt".to_string()),
        ];
        assert_eq!(tokens, expected_tokens);
        let parts = parse_command_chain(tokens).unwrap();
        assert_eq!(parts.len(), 1);
        let CommandPart::Execute { name, args, stdin, stdout } = &parts[0];

        assert_eq!(name, "echo");
        assert_eq!(args, &vec!["123".to_string()]);
        assert_eq!(*stdin, ExecutionSource::Inherit);
        assert_eq!(*stdout, ExecutionSource::File(">>a.txt".to_string()));
    }

    #[test]
    fn test_token_pipe() {
        let input = "echo 123 | cat";
        let tokens = tokenize(input);
        let expected_tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("123".to_string()),
            Token::Pipe,
            Token::Word("cat".to_string()),
        ];
        assert_eq!(tokens, expected_tokens);
        let parts = parse_command_chain(tokens).unwrap();
        assert_eq!(parts.len(), 2);
        let CommandPart::Execute { name, args, stdin, stdout } = &parts[0];
        assert_eq!(name, "echo");
        assert_eq!(args, &vec!["123".to_string()]);
        assert_eq!(*stdin, ExecutionSource::Inherit);
        assert_eq!(*stdout, ExecutionSource::Pipe(PipeEndpoint::Write));
        let CommandPart::Execute { name, args, stdin, stdout } = &parts[1];
        assert_eq!(name, "cat");
        assert!(args.is_empty());
        assert_eq!(*stdin, ExecutionSource::Pipe(PipeEndpoint::Read));
        assert_eq!(*stdout, ExecutionSource::Inherit);
    }
}