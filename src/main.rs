mod commands_execution;
mod command;
mod lexer;
mod env;
mod completer;
mod trie;
mod shell;
mod output_redirection;
mod program;
mod jobs;

use shell::Shell;

fn main(){
    match Shell::new(){
        Ok(mut shell) => shell.run(),
        Err(err) => {
            eprintln!("Unable to start the shell: {}", err);
        }
    }
}
