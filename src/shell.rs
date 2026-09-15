use std::collections::HashMap;

use crate::jobs::{Job};
use crate::lexer::{Token};
use crate::{completer, lexer};
use crate::command_pipeline::{CommandPipeline};
use rustyline::{ CompletionType, Config, history::DefaultHistory };
use rustyline::error::{ReadlineError};

type Editor = rustyline::Editor<completer::Completer, DefaultHistory>;

pub struct Shell{
    pub current_job_id: u32,
    pub editor: Editor,
    pub jobs: Vec<Job>,
    pub variables: HashMap<String, String>
}

impl Shell{
    fn new_editor() -> Result<Editor, String>{
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut editor = Editor::with_config(config)
            .map_err(|err| format!("{err}"))?;

        editor.set_helper(Some(completer::Completer::new()));

        Ok(editor)
    }

    pub fn new() -> Result<Self, String>{
        Ok(Self{
            current_job_id: 0,
            editor: Self::new_editor()?,
            jobs: Vec::with_capacity(32),
            variables: HashMap::new()
        })
    }

    pub fn run(&mut self){
        let history_file = std::env::var("HISTFILE");

        if let Ok(history_file) = history_file.clone(){
            let _ = self.editor.load_history(&history_file);
        }

        loop{
            let input = self.editor.readline("$ ");

            match input{
                Ok(input) => {
                    let input = input.trim();
                    let _ = self.editor.add_history_entry(input);

                    if input.is_empty(){
                        continue;
                    }

                    match lexer::CommandLine::from(input).map(|mut cmd| { 
                        for cmd in &mut cmd[..]{
                            let Token::Arg(arg) = cmd else { continue; };
                            *arg = self.insert_variables_values(arg.clone());
                        }

                        cmd.retain(|e| *e != Token::Arg("".to_owned()));

                        CommandPipeline::new(&cmd) 
                    }){
                        Ok(Ok(command)) => {
                            if self.execute_cmd_pipeline(command).should_exit(){
                                break;
                            }
                        },
                        Err(err) | Ok(Err(err)) => {
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

        if let Ok(history_file) = history_file{
            let _ = self.write_history(&history_file);
        }
    }

    
}
