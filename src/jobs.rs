use std::process::{Child};
use strum_macros::EnumString;
use strum_macros::Display;

use crate::shell::Shell;

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
