use crate::{completer};
use crate::command::{Command};
use rustyline::{ CompletionType, Config, history::DefaultHistory };
use rustyline::error::{ReadlineError};

type Editor = rustyline::Editor<completer::Completer, DefaultHistory>;

pub struct Job{
    pub pid: u32,
    pub command: String
}

pub struct Shell{
    pub editor: Editor,
    pub jobs: Vec<Job>
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
            editor: Self::new_editor()?,
            jobs: Vec::with_capacity(32)
        })
    }

    pub fn run(&mut self){
        loop{
            let input = self.editor.readline("$ ");

            match input{
                Ok(input) => {
                    let input = input.trim();
                    let _ = self.editor.add_history_entry(input);

                    if input.is_empty(){
                        continue;
                    }

                    match Command::from_str(input){
                        Ok(command) => {
                            if self.execute_command(command).should_exit(){
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
}
