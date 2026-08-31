#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

mod commands_execution;
mod command;
mod parser;
mod env;

use command::{ Command, CommandArgs };

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input error. Exiting");
        let input = input.trim();

        let input = parser::remove_extra_spaces_outside_quotes(input);
        if input.is_none(){
            println!("Missing closing quote");
            continue;
        }

        let input = input.unwrap();

        let command = parser::split_command(&input);
        let command_name = command.first().unwrap();
        let command_args = &command[1..];

        let command = Command::from(command_name, command_args);

        if command.is_none(){
            if execute_process(command_name, command_args).is_none(){
                println!("{}: command not found", command_name);
            }
            continue;
        }

        if command.unwrap().execute().should_exit(){
            return;
        }
    }
}

fn execute_process(executable: &str, args: CommandArgs) -> Option<Result<process::ExitStatus, std::io::Error>>{
    env::find_in_path(executable).as_ref()?;

    let mut command = process::Command::new(executable);
    if !args.is_empty(){
        command.args(args);
    }

    Some(match command.spawn().and_then(|mut c| c.wait()){
        Ok(status) => Ok(status),
        Err(err) => {
            eprintln!("Failed to execute the process: {}", err);
            Err(err)
        }
    })
}


