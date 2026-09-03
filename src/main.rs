#[allow(unused_imports)]
use std::io::{self, Write};

mod commands_execution;
mod command;
mod lexer;
mod env;

use command::{ Command };

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input error. Exiting");
        let input = input.trim();

        if input.is_empty(){
            continue;
        }

        match Command::from_str(input){
            Ok(command) => {
                if command.execute().should_exit(){
                    return;
                }
            },
            Err(err) => {
                println!("Syntax Error: {err}");
            }
        }
    }
}
