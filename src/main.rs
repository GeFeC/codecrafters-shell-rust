#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use is_executable::is_executable;
use std::process;

enum Command<'a>{
    Exit,
    Pwd,
    Echo(&'a str),
    Type(&'a str),
    Cd(&'a str)
}

impl Command<'_>{
    fn from<'a>(command: &'a str, rest: &'a str) -> Option<Command<'a>>{
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

    println!("{} is a shell builtin", cmd_name);
}

fn execute_process(filepath: &str, args: &str) -> Result<process::ExitStatus, std::io::Error>{
    let mut command = process::Command::new(filepath);
    let cmd_result = match args{
        "" => command.spawn(),
        _ => command 
            .args(args.split(' ').collect::<Vec<_>>())
            .spawn()
    };

    cmd_result?.wait()
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
            Command::Echo(text) => { println!("{}", text) }
            Command::Type(checked) => { handle_type_command(checked); }
            Command::Pwd => println!("{}", env::current_dir().unwrap().display()),
            Command::Cd(path) => {
                let path = match path{
                    "~" | "" => match env::home_dir(){
                        Some(home) => home.to_str().unwrap().to_owned(),
                        None => String::from("~")
                    },
                    _ => String::from(path)
                };

                env::set_current_dir(&path)
                    .unwrap_or_else(|_| println!("cd: {}: No such file or directory", path));
            }
        }
    }
}
