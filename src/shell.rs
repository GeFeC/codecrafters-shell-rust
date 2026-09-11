use std::process::{Child};

use crate::{completer, lexer};
use crate::command::{Command};
use rustyline::{ CompletionType, Config, history::DefaultHistory };
use rustyline::error::{ReadlineError};
use strum_macros::EnumString;
use strum_macros::Display;
use std::string::ToString;

type Editor = rustyline::Editor<completer::Completer, DefaultHistory>;

#[derive(EnumString, Display)]
pub enum JobState{
    #[strum(to_string = "Running")]
    Running,
    #[strum(to_string = "Done")]
    Done
}

pub struct Job{
    pub idx: u32,
    pub state: JobState,
    pub process: Child,
    pub command: String
}

impl Job{
    pub fn display(&self, symbol: char) -> String{
        format!("[{}]{}  {:24}{}\n", self.idx, symbol, self.state.to_string(), self.command)
    }
}

pub struct Shell{
    current_job_id: u32,
    pub editor: Editor,
    pub jobs: Vec<Job>
}

impl Shell{
    pub fn new_job(&mut self, process: Child, command: String) -> Job{
        self.current_job_id = self.jobs.iter().map(|j| j.idx).max().unwrap_or(0);

        self.current_job_id += 1;
        
        Job{
            idx: self.current_job_id,
            state: JobState::Running,
            process,
            command
        }
    }

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

    pub fn remove_finished_jobs(&mut self){
        self.jobs.retain(|j| !matches!(j.state, JobState::Done));
    }

    pub fn update_jobs(&mut self){
        for job in &mut self.jobs{
            job.state = match job.process.try_wait(){
                Ok(Some(_)) => JobState::Done,
                Ok(None) => JobState::Running,
                Err(_err) => todo!()
            };
        }
    }

    pub fn display_jobs(&mut self, filter: impl FnMut(&(usize, &mut Job)) -> bool) -> String{
        let total_jobs = self.jobs.len();

        let output = self.jobs.iter_mut()
            .enumerate()
            .filter(filter)
            .map(|(idx, job)| {
                let symbol = match idx{
                    _ if idx + 1 == total_jobs => '+',
                    _ if idx + 2 == total_jobs => '-',
                    _ => ' '
                };

                job.display(symbol)
            })
            .collect::<Vec<_>>()
            .join("");

        output.trim_end().to_string()
    }
}
