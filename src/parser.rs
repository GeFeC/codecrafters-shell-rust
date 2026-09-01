use crate::command;

pub fn split_command(args: &str) -> Option<command::CommandArgsStorage>{
    let mut result = command::CommandArgsStorage::new();

    let mut arg = String::with_capacity(args.len());
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;

    let mut iter = args.chars();

    while let Some(c) = iter.next(){
        if c == '\\' && !in_single_quotes && let Some(c) = iter.next(){
            arg.push(c);
            continue;
        }

        if c == '"' && !in_single_quotes{
            in_double_quotes = !in_double_quotes;
            continue;
        }

        if c == '\'' && !in_double_quotes{
            in_single_quotes = !in_single_quotes;
            continue;
        }

        let in_quotes = in_single_quotes || in_double_quotes;

        if c == ' ' && !in_quotes{
            if !arg.is_empty(){
                result.push(arg);
                arg = "".to_string();
            }
            continue;
        }

        arg.push(c);
    }

    if in_single_quotes || in_double_quotes{
        return None;
    }

    if !arg.is_empty(){
        result.push(arg);
    }

    Some(result)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_missing_closing_quote(){
        assert!(split_command("\"hello world").is_none());
        assert!(split_command("\'hello world").is_none());
        assert!(split_command("\"shell's test\"").is_some());
        assert!(split_command("'shell\"s test'").is_some());
    }

    #[test]
    fn test_extra_spaces_deletion(){
        let cases = [
            ("hello   world", "hello world"),
            ("\"hello   world\"", "hello   world"),
            ("'hello   world'    hello   world", "hello   world hello world"),
            ("  hello   \"hello   world\"     world", "hello hello   world world"),
        ];

        for (case, expected) in cases{
            let got = split_command(case);
            assert!(got.is_some());
            let got = got.unwrap().join(" ");

            assert_eq!(got, expected);
        }
    }

    #[test]
    fn test_quotes(){
        let cases = [
            ("'hello   world'", vec!["hello   world"]),
            ("hello   world", vec!["hello", "world"]),
            ("'hello''world'", vec!["helloworld"]),
            ("hello''world", vec!["helloworld"]),
            ("hello\"\"world    ", vec!["helloworld"])
        ];

        for (case, expected) in cases{
            assert_eq!(split_command(case).unwrap(), expected);
        }
    }

    #[test]
    fn test_escape_character(){
        let cases = [
            (r#"three\ \ \ spaces"#, vec!["three   spaces"]),
            (r#"before\    after"#, vec!["before ", "after"]),
            (r#"test\nexample"#, vec!["testnexample"]),
            (r#"hello\\world"#, vec![r#"hello\world"#]),
            (r#"\'hello\'"#, vec!["'hello'"]),
        ];

        for (case, expected) in cases{
            assert_eq!(split_command(case).unwrap(), expected);
        }
    }
    
    #[test]
    fn test_escape_character_inside_single_quotes(){
        let cases = [
            (r#"'shell\\\nscript'"#, vec![r#"shell\\\nscript"#]),
            (r#"'every\"thing_is\"literal'"#, vec![r#"every\"thing_is\"literal"#]),
        ];

        for (case, expected) in cases{
            assert_eq!(split_command(case).unwrap(), expected);
        }
    }
}
