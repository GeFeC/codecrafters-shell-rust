#[allow(unused_imports)]
use std::io::{self, Write};

enum CommandType{
    Invalid,
    ShellBuiltin
}

enum CmdVariant<'a>{
    Invalid,
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
    fn from_prompt(prompt: &str) -> Command<'_>{
        let (command, params) = prompt.split_once(' ').unwrap_or((prompt, ""));
        match command{
            "exit" => Command{ 
                name: command, 
                variant: CmdVariant::Exit, 
                r#type: CommandType::ShellBuiltin 
            },
            "echo" => Command{ 
                name: command, 
                variant: CmdVariant::Echo(params), 
                r#type: CommandType::ShellBuiltin 
            },
            "type" => Command{ 
                name: command, 
                variant: CmdVariant::Type(params), 
                r#type: CommandType::ShellBuiltin 
            },
            _ => Command{ name: "", variant: CmdVariant::Invalid, r#type: CommandType::Invalid }
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
        let command = Command::from_prompt(input);

        match command.variant{
            CmdVariant::Invalid => println!("{}: command not found", command.name),
            CmdVariant::Exit => return,
            CmdVariant::Echo(text) => { println!("{}", text) }
            CmdVariant::Type(cmd_name) => {
                let checked_command = Command::from_prompt(cmd_name);

                match checked_command.r#type{
                    CommandType::Invalid => {
                        println!("{}: not found", cmd_name);
                    }
                    CommandType::ShellBuiltin => {
                        println!("{} is a shell builtin", cmd_name);
                    }
                }
            }
        }
    }
}
