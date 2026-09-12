use core::fmt;
use std::ops::Deref;

#[derive(Debug)]
pub struct CommandLine(pub Vec<Token>);

impl Deref for CommandLine{
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl CommandLine{
    pub fn from(args: &str) -> Result<CommandLine, String>{
        let mut result = Vec::<Token>::new();

        let mut arg = String::with_capacity(args.len());
        let in_single_quotes = false;
        let in_double_quotes = false;

        let args = args.trim();
        let mut iter = args.char_indices().chain(std::iter::once((0, '\0')));

        while let Some((_, c)) = iter.next(){
            match c{
                _ if TERMINATORS.contains(&c) => {
                    /*
                     * When terminator is lexed, arg can contain invalid data such as
                     * an empty string, or a part of the redirection operator. We don't want that to be
                     * pushed to the token array
                     * */
                    let arg_contains_valid_data = !(c == '>' && ["1", "2"].contains(&arg.as_str())) && !arg.is_empty();

                    if arg_contains_valid_data{
                        result.push(Token::Arg(arg.clone()));
                    }
                    arg.clear();

                    match c{
                        ' ' => { },
                        '|' => {
                            result.push(Token::Pipe);
                        }
                        '>' => {
                            let (token, new_iter) = lexe_redirection(args, iter)?;
                            iter = new_iter;
                            result.push(token);
                        }
                        '\0' => {}
                        _ => unreachable!()
                    };
                    iter = skip_spaces(iter);
                }
                c => {
                    let (new_arg, new_iter) = match c{
                        '"' | '\'' => lexe_arg_inside_quoutes(iter, c)?,
                        _ => lexe_arg_outside_quotes(c, iter)?
                    };
                    iter = new_iter;
                    arg.push_str(&new_arg);
                }
            }
        }

        if in_single_quotes || in_double_quotes{
            return Err("Missing closing quote".to_owned());
        }

        Ok(CommandLine(result))
    }
}

impl std::string::ToString for CommandLine{
    fn to_string(&self) -> String{
        self.0.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" ")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token{
    Arg(String),
    Stdout,
    Stderr,
    StdoutAppend,
    StderrAppend,
    Pipe
}

impl Token{
    pub fn is_redirection(&self) -> bool{
        matches!(self, Token::Stdout | Token::Stderr | Token::StdoutAppend | Token::StderrAppend)
    }
}


impl std::fmt::Display for Token{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result{
        write!(f, "{}", match self{
            Self::Arg(arg) => arg.clone(),
            Self::Stdout => String::from(">"),
            Self::Stderr => String::from("2>"),
            Self::StdoutAppend => String::from(">>"),
            Self::StderrAppend => String::from("2>>"),
            Self::Pipe => String::from("|")
        })
    }
}

pub trait LexerIterator: Iterator<Item = (usize, char)> + Clone {}
impl<T> LexerIterator for T where T: Iterator<Item = (usize, char)> + Clone {}

fn lexe_arg_outside_quotes<I: LexerIterator>(first: char, mut iter: I) -> Result<(String, I), String>{
    let mut arg = String::with_capacity(32);

    let next_char_terminates = |iter: &I|{
        let (_, next) = iter.clone().next().unwrap();
        if TERMINATORS.contains(&next){ return true; };
        if QUOTES.contains(&next){ return true; };
        false
    };

    if first != '\\'{
        arg.push(first);
        if next_char_terminates(&iter) { return Ok((arg, iter)); }
    }

    while let Some((_, c)) = iter.next(){
        match c{
            '\\' => match iter.next(){
                Some((_, '\0')) => {
                    return Err("Unexpected end of string after escape character".to_owned())
                }
                Some((_, c)) => { arg.push(c); },
                None => unreachable!()
            }
            _ => {
                arg.push(c);
            }
        }

        if next_char_terminates(&iter){ break; }
    }

    Ok((arg, iter))
}

fn lexe_arg_inside_quoutes<I: LexerIterator>(mut iter: I, termination: char) -> Result<(String, I), String>{
    let mut arg = String::with_capacity(iter.clone().count());
    while let Some((_, c)) = iter.next(){
        match c{
            '\\' if termination == '"' && let Some((_, c)) = iter.next() => {
                let should_escape = ['"', '\\', '$', '`', '\n'].contains(&c);

                if should_escape{
                    arg.push(c);
                    continue;
                }

                arg.push('\\');
                arg.push(c);
            },
            _ if c == termination => {
                return Ok((arg, iter));
            },
            _ => {
                arg.push(c);
            }
        }
    }

    Err("Missing closing quote".to_string())
}

fn skip_spaces<I: LexerIterator>(mut iter: I) -> I{
    while let Some((_, ' ')) = iter.clone().next(){
        iter.next();
    }

    iter
}

fn lexe_redirection<I: LexerIterator>(args: &str, mut iter: I) -> Result<(Token, I), String>{
    let next = iter.clone().next();

    if let Some((_, '\0')) = next{
        return Err("No target file provided".to_owned());
    }

    let (next_idx, next_c) = next.unwrap();
    let idx = next_idx - 1;

    let prev_two_characters = if idx >= 2{ 
        &args[(idx - 2)..(idx)] 
    } 
    else { "" };
    
    let second_symbol = if next_c == '>'{ ">" } else{ "" };

    if second_symbol == ">"{
        iter.next();
    }

    let symbol = match prev_two_characters{
        " 2" => "2>",
        _ => ">"
    };

    Ok((match format!("{symbol}{second_symbol}").as_str(){
        ">" => Token::Stdout,
        "2>" => Token::Stderr,
        ">>" => Token::StdoutAppend,
        "2>>" => Token::StderrAppend,
        _ => unreachable!()
    }, iter))
}

const TERMINATORS: &[char] = &[' ', '>', '\0', '|'];
const QUOTES: &[char] = &['\'', '"'];

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_missing_closing_quote(){
        assert!(CommandLine::from("\"hello world").is_err());
        assert!(CommandLine::from("\'hello world").is_err());
        assert!(CommandLine::from("\"shell's test\"").is_ok());
        assert!(CommandLine::from("'shell\"s test'").is_ok());
    }

    fn arg(str: &str) -> Token{
        Token::Arg(str.to_owned())
    }

    fn test(cases: &[(&str, Vec<Token>)]){
        for (idx, (case, expected)) in cases.iter().enumerate(){
            assert_eq!(CommandLine::from(case).unwrap().0, *expected, "Case #{} Failed", idx + 1);
        }
    }

    #[test]
    fn test_extra_spaces_deletion(){
        test(&[
            ("hello   world", vec![arg("hello"), arg("world")]),
            ("\"hello   world\"", vec![arg("hello   world")]),
            ("'hello   world'    hello   world", vec![arg("hello   world"), arg("hello"), arg("world")]),
            ("  hello   \"hello   world\"     world", vec![arg("hello"), arg("hello   world"), arg("world")]),
        ]);
    }

    #[test]
    fn test_quotes(){
        test(&[
            ("'hello   world'", vec![arg("hello   world")]),
            ("hello   world", vec![arg("hello"), arg("world")]),
            ("'hello''world'", vec![arg("helloworld")]),
            ("hello''world", vec![arg("helloworld")]),
            ("hello\"\"world    ", vec![arg("helloworld")])
        ]);
    }

    #[test]
    fn test_escape_character(){
        test(&[
            (r#"three\ \ \ spaces"#, vec![arg("three   spaces")]),
            (r#"before\    after"#, vec![arg("before "), arg("after")]),
            (r#"test\nexample"#, vec![arg("testnexample")]),
            (r#"hello\\world"#, vec![arg(r#"hello\world"#)]),
            (r#"\'hello\'"#, vec![arg(r#"'hello'"#)]),
        ]);

        assert!(CommandLine::from("hello\\").is_err());
    }

    #[test]
    fn test_escape_character_inside_single_quotes(){
        test(&[
            (r#"'shell\\\nscript'"#, vec![arg(r#"shell\\\nscript"#)]),
            (r#"'every\"thing_is\"literal'"#, vec![arg(r#"every\"thing_is\"literal"#)]),
        ]);
    }

    #[test]
    fn test_escape_character_inside_double_quotes(){
        test(&[
            (r#""A \\ escapes itself""#, vec![arg(r#"A \ escapes itself"#)]),
            (r#""A \" inside double quotes""#, vec![arg(r#"A " inside double quotes"#)]),
            (r#""hello\ ""#, vec![arg(r#"hello\ "#)])
        ]);
    }

    #[test]
    fn test_stdout_redirection(){
        test(&[
            (r#"hello>world"#, vec![arg("hello"), Token::Stdout, arg("world")]),
            (r#"a>b"#, vec![arg("a"), Token::Stdout, arg("b")]),
            (r#"a 1>b"#, vec![arg("a"), Token::Stdout, arg("b")]),
            (r#"hello1>world"#, vec![arg("hello1"), Token::Stdout, arg("world")]),
            (r#"hello 1> world"#, vec![arg("hello"), Token::Stdout, arg("world")]),
            (r#"hello > world"#, vec![arg("hello"), Token::Stdout, arg("world")]),
            (r#""hello > world""#, vec![arg("hello > world")]),
        ]);

        assert!(CommandLine::from("hello>").is_err());
    }

    #[test]
    fn test_stderr_redirection(){
        test(&[
            (r#"hello2>world"#, vec![arg("hello2"), Token::Stdout, arg("world")]),
            (r#""hello"2>world"#, vec![arg("hello2"), Token::Stdout, arg("world")]),
            (r#"hello 2> world"#, vec![arg("hello"), Token::Stderr, arg("world")]),
            (r#""hello 2> world""#, vec![arg("hello 2> world")]),
            (r#"a 2>b"#, vec![arg("a"), Token::Stderr, arg("b")]),
        ]);
    }

    #[test]
    fn test_stdout_redirection_in_append_mode(){
        test(&[
            (r#"hello>>world"#, vec![arg("hello"), Token::StdoutAppend, arg("world")]),
            (r#"hello1>>world"#, vec![arg("hello1"), Token::StdoutAppend, arg("world")]),
            (r#"hello 1>> world"#, vec![arg("hello"), Token::StdoutAppend, arg("world")]),
            (r#"hello >> world"#, vec![arg("hello"), Token::StdoutAppend, arg("world")]),
            (r#""hello >> world""#, vec![arg("hello >> world")]),
        ]);
    }

    #[test]
    fn test_stderr_redirection_in_append_mode(){
        test(&[
            (r#"hello2>>world"#, vec![arg("hello2"), Token::StdoutAppend, arg("world")]),
            (r#""hello"2>>world"#, vec![arg("hello2"), Token::StdoutAppend, arg("world")]),
            (r#"hello 2>> world"#, vec![arg("hello"), Token::StderrAppend, arg("world")]),
            (r#""hello 2>> world""#, vec![arg("hello 2>> world")]),
        ]);
    }

    #[test]
    fn test_pipe(){
        test(&[
            (r#"hello | world"#, vec![arg("hello"), Token::Pipe, arg("world")]),
            (r#"echo "hello | pipes" | wc"#, vec![arg("echo"), arg("hello | pipes"), Token::Pipe, arg("wc")]),
        ]);
    }
}
