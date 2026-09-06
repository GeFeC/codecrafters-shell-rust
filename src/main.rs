#[allow(unused_imports)]
use std::io::{self, Write};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

mod commands_execution;
mod command;
mod lexer;
mod env;
mod completer;
mod trie;

use command::{ Command };

fn main(){
    let mut rl = Editor::<completer::CommandCompleter, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(completer::CommandCompleter::new()));
    loop{
        let input = rl.readline("$ ");

        match input{
            Ok(input) => {
                let input = input.trim();
                let _ = rl.add_history_entry(input);

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
                        println!("{err}");
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
            },
            Err(err) => {
                println!("Input error: {err}");
            }
        }
    }
}
