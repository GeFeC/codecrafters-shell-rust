use crate::command;

//Returns None if quotes parity isn't met
pub fn remove_extra_spaces_outside_quotes(string: &str) -> Option<String>{ 
    let mut in_single_quotes = false;
    let mut in_double_quotes = false;
    let mut last_was_space = false;
    let mut result = String::with_capacity(string.len());

    for c in string.chars(){
        if c == '"' && !in_single_quotes{
            in_double_quotes = !in_double_quotes;
        }

        if c == '\'' && !in_double_quotes{
            in_single_quotes = !in_single_quotes;
        }

        let is_space = c == ' ';
        let in_quotes = in_single_quotes || in_double_quotes;

        if in_quotes || !last_was_space || !is_space{
            result.push(c);
        }

        last_was_space = !in_quotes && is_space;
    }

    if !in_single_quotes && !in_double_quotes{
        return Some(result);
    }

    None
}

pub fn split_command<'a>(args: &'a str) -> command::CommandArgsStorage<'a>{
    let mut args = args;
    let mut result = Vec::<&str>::new();

    while !args.is_empty() {
        for (idx, c) in args.char_indices(){
            match c{
                quote @ ('\'' | '"') => {
                    args = &args[idx..];
                    let opening_quote_pos = 0;

                    let closing_quote_pos = args[(opening_quote_pos + 1)..].find(quote).unwrap() + 1;

                    result.push(&args[(opening_quote_pos + 1)..=(closing_quote_pos - 1)]);

                    args = &args[(closing_quote_pos + 1)..];
                    break;
                },
                ' ' => {
                    if idx == 0{
                        args = &args[1..];
                        break;
                    }
                    
                    result.push(&args[0..idx]);
                    args = &args[(idx + 1)..];
                    break;
                },
                _ => {
                    if idx == args.chars().count() - 1{
                        result.push(args);
                        args = "";
                        break;
                    }
                }
            }
        }
    }
    result
}

