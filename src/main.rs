#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use is_executable::is_executable;
use std::process;
use std::path::Path;

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input error. Exiting");
        let input = input.trim();

        if !check_quotes_parity(input){
            println!("Missing closing quote");
            continue;
        }

        let input = remove_extra_spaces_outside_quotes(input);

        let (command_name, rest) = split_prompt(&input);
        let command_args = split_command_args(rest);

        let command = Command::from(command_name, &command_args);

        if command.is_none(){
            if find_in_env_path(command_name).is_some(){
                if let Err(err) = execute_process(command_name, rest){
                    eprintln!("Failed to execute the process: {}", err);
                }
                continue;
            }

            println!("{}: command not found", command_name);
            continue;
        }

        match command.unwrap(){
            Command::Exit => return,
            Command::Echo(text) => { println!("{}", text.join(" ")) }
            Command::Type(checked) => { handle_type_command(checked); }
            Command::Pwd => println!("{}", env::current_dir().unwrap().display()),
            Command::Cd(path) => {
                let home_dir = env::home_dir();

                let path = path.join(" ");
                let path = match path.as_str(){
                    "~" | "" => home_dir.as_deref().and_then(Path::to_str).unwrap_or("~").to_owned(),
                    _ => path
                };

                env::set_current_dir(&path)
                    .unwrap_or_else(|_| println!("cd: {}: No such file or directory", path));
            }
        }
    }
}

fn check_quotes_parity(string: &str) -> bool{
    let mut single_quote = false;
    let mut double_quote = false;

    for c in string.chars(){
        if c == '"' && !single_quote{
            double_quote = !double_quote;
        }

        if c == '\'' && !double_quote{
            single_quote = !single_quote;
        }
    }

    !single_quote && !double_quote
}

fn remove_extra_spaces_outside_quotes(string: &str) -> String{
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

        last_was_space = !in_single_quotes && is_space;
    }

    result
}

type CommandArgs<'a> = &'a[&'a str];

enum Command<'a>{
    Exit,
    Pwd,
    Echo(CommandArgs<'a>),
    Type(CommandArgs<'a>),
    Cd(CommandArgs<'a>)
}

impl Command<'_>{
    fn from<'a>(command: &'a str, rest: CommandArgs<'a>) -> Option<Command<'a>>{
        match command{
            "exit" => Some(Command::Exit),
            "echo" => Some(Command::Echo(rest)),
            "type" => Some(Command::Type(rest)),
            "pwd" => Some(Command::Pwd),
            "cd" => Some(Command::Cd(rest)),
            _ => None
        }
    }
}

fn split_prompt(prompt: &str) -> (&str, &str){
    prompt
        .split_once(' ')
        .unwrap_or((prompt, ""))
}

fn find_in_env_path(file: &str) -> Option<String>{
    let path_env = env::var("PATH").expect("Unable to read $PATH variable");
    let path_dirs = env::split_paths(&path_env);

    for dir in path_dirs{
        let paths = fs::read_dir(dir);

        if paths.is_err(){
            continue;
        }

        for path in paths.unwrap(){
            let path = path.ok()?.path();
            if path.is_file() && is_executable(&path) && path.file_name()?.to_str()? == file{
                return Some(path.to_str()?.to_owned());
            }
        }
    }

    None
}

fn handle_type_command(checked_cmd: CommandArgs){
    if checked_cmd.len() != 1{
        println!("Usage: type <command name without args>");
        return;
    }

    let cmd_name = checked_cmd.first().unwrap();
    let command = Command::from(cmd_name, &[]);

    if command.is_none(){
        let executable_path = find_in_env_path(cmd_name);

        match executable_path{
            Some(path) => println!("{} is {}", cmd_name, path),
            None => {
                println!("{}: not found", cmd_name);
            }
        }

        return;
    }

    println!("{} is a shell builtin", cmd_name);
}

fn execute_process(filepath: &str, args: &str) -> Result<process::ExitStatus, std::io::Error>{
    let mut command = process::Command::new(filepath);
    let cmd_result = match args{
        "" => command.spawn(),
        _ => command 
            .args(split_command_args(args))
            .spawn()
    };

    cmd_result?.wait()
}

fn split_command_args(args: &str) -> Vec<&str>{
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
