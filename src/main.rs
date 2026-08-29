#[allow(unused_imports)]
use std::io::{self, Write};

enum Command<'a>{
    Exit,
    Echo(&'a str)
}

impl Command<'_>{
    fn from_str(prompt: &str) -> Option<Command<'_>>{
        let (command, params) = prompt.split_once(' ').unwrap_or((prompt, ""));
        match command{
            "exit" => Some(Command::Exit),
            "echo" => Some(Command::Echo(params)),
            _ => None
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
        }
    }
}
