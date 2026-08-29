#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use is_executable::is_executable;

enum CommandType{
    ShellBuiltin
}

enum CmdVariant<'a>{
    Exit,
    Echo(&'a str),
    Type(&'a str)
}

struct Command<'a>{
    name: &'a str,
    variant: CmdVariant<'a>,
    r#type: CommandType
}

impl Command<'_>{
    fn from<'a>(command: &'a str, rest: &'a str) -> Option<Command<'a>>{
        match command{
            "exit" => Some(Command{ 
                name: command, 
                variant: CmdVariant::Exit, 
                r#type: CommandType::ShellBuiltin 
            }),
            "echo" => Some(Command{ 
                name: command, 
                variant: CmdVariant::Echo(rest), 
                r#type: CommandType::ShellBuiltin 
            }),
            "type" => Some(Command{ 
                name: command, 
                variant: CmdVariant::Type(rest), 
                r#type: CommandType::ShellBuiltin 
            }),
            _ => None
        }
    }
}

fn split_prompt(prompt: &str) -> (&str, &str){
    prompt.split_once(' ').unwrap_or((prompt, ""))
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

fn handle_type_command(checked_cmd: &str){
    let (cmd_name, _) = split_prompt(checked_cmd);
    let command = Command::from(cmd_name, "");

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

    match command.unwrap().r#type{
        CommandType::ShellBuiltin => {
            println!("{} is a shell builtin", cmd_name);
        }
    }
}

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input error. Exiting");
        let input = input.trim();
        let (command_name, rest) = split_prompt(input);
        let command = Command::from(command_name, rest);

        if command.is_none(){
            println!("{}: command not found", command_name);
            continue;
        }

        match command.unwrap().variant{
            CmdVariant::Exit => return,
            CmdVariant::Echo(text) => { println!("{}", text) }
            CmdVariant::Type(checked) => { handle_type_command(checked); }
        }
    }
}
