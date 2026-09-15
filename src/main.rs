mod command;
mod lexer;
mod env;
mod completer;
mod trie;
mod shell;
mod output_redirection;
mod program;
mod jobs;
mod command_pipeline;
mod history;
mod variables;

use shell::Shell;

fn main(){
    match Shell::new(){
        Ok(mut shell) => shell.run(),
        Err(err) => {
            eprintln!("Unable to start the shell: {}", err);
        }
    }
}
