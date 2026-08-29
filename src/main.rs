#[allow(unused_imports)]
use std::io::{self, Write};

enum CommandType{
    ShellBuiltin
}

enum Command<'a>{
    Exit,
    Echo(&'a str),
    Type(&'a str)
}

impl Command<'_>{
    fn from_str(prompt: &str) -> Option<Command<'_>>{
        let (command, params) = prompt.split_once(' ').unwrap_or((prompt, ""));
        match command{
            "exit" => Some(Command::Exit),
            "echo" => Some(Command::Echo(params)),
            "type" => Some(Command::Type(params)),
            _ => None
        }
    }

    fn get_type(&self) -> CommandType{
        match self{
            Command::Exit => CommandType::ShellBuiltin,
            Command::Echo(_) => CommandType::ShellBuiltin,
            Command::Type(_) => CommandType::ShellBuiltin
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
        let command = Command::from_str(input);

        if command.is_none(){
            println!("{}: command not found", input);
            continue;
        }

        match command.unwrap(){
            Command::Exit => return,
            Command::Echo(text) => { println!("{}", text) }
            Command::Type(cmd_name) => {
                let checked_command = Command::from_str(cmd_name);
                if checked_command.is_none(){
                    println!("{}: not found", cmd_name);
                    continue;
                }

                match checked_command.unwrap().get_type(){
                    CommandType::ShellBuiltin => {
                        println!("{} is a shell builtin", cmd_name);
                    }
                }
            }
        }
    }
}
