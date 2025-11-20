use std::env;

pub fn expand_env_vars(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some('$') = chars.peek().copied() {
                    chars.next();
                    out.push('$');
                } else {
                    out.push('\\');
                }
            }
            '$' => {
                // Handle ${VAR} or $VAR
                if let Some('{') = chars.peek().copied() {
                    // ${VAR}
                    chars.next(); // consume '{'
                    let mut name = String::new();
                    while let Some(nc) = chars.next() {
                        if nc == '}' {
                            break;
                        }
                        name.push(nc);
                    }
                    let val = env::var(&name).unwrap_or_default();
                    out.push_str(&val);
                } else if let Some('$') = chars.peek().copied()
                {
                    // $$ -> PID
                    chars.next(); // consume second '$'
                    let pid = std::process::id();
                    out.push_str(&pid.to_string());
                } else if let Some(digit @ ('0'..='9')) = chars.peek().copied() {
                    chars.next();
                    let mut script_name = String::new();
                    // $0 -> script name
                    if digit == '0' {
                        script_name = env::args().next().unwrap_or_default();
                    }
                    out.push_str(&script_name);
                } else {
                    // $VAR
                    let mut name = String::new();
                    // First char must be [A-Za-z_]
                    if let Some(nc) = chars.peek().copied() {
                        if nc.is_ascii_alphabetic() || nc == '_' {
                            name.push(nc);
                            chars.next();
                            while let Some(nc2) = chars.peek().copied() {
                                if nc2.is_ascii_alphanumeric() || nc2 == '_' {
                                    name.push(nc2);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            let val = env::var(&name).unwrap_or_default();
                            out.push_str(&val);
                        } else {
                            // Not a valid var name, keep '$'
                            out.push('$');
                        }
                    } else {
                        // '$' at end
                        out.push('$');
                    }
                }
            }
            _ => out.push(c),
        }
    }

    out
}


#[cfg(test)]
mod tests {
    use crate::token::{tokenize, Token};

    #[test]
    fn test_env_expand_basic() {
        unsafe {
            std::env::set_var("FOO_TEST", "hello");
        }
        let input = "echo $FOO_TEST";
        let tokens = tokenize(input);
        let expected_tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("hello".to_string()),
        ];
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    fn test_env_expand_braced_and_escape() {
        unsafe {
            std::env::set_var("BAR_TEST", "world");
        }
        let input = "echo ${BAR_TEST} \\$BAR_TEST \\$$BAR_TEST";
        let tokens = tokenize(input);
        let expected_tokens = vec![
            Token::Word("echo".to_string()),
            Token::Word("world".to_string()),
            Token::Word("$BAR_TEST".to_string()),
            Token::Word("$world".to_string()),
        ];
        assert_eq!(tokens, expected_tokens);
    }
}