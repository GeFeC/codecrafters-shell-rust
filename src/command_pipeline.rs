use std::fs::OpenOptions;
use std::process::Stdio;
use std::string;
use std::thread::current;

use crate::command::{ExecutionStatus, Output};
use crate::output_redirection::{Redirection, RedirectionTarget, RedirectionType};

use std::io::Write;
use crate::lexer::{CommandLine, Token};
use crate::command::{Command, ExecutionPolicy};
use crate::command;
use crate::program::Program;
use crate::shell::Shell;

pub struct CommandPipeline{
    raw: CommandLine,
    pub first_command: Option<Command>,
    pub processes: Vec<Program>,
    pub execution_policy: ExecutionPolicy,
    pub redirection: Option<Redirection>
}

impl CommandPipeline{
    pub fn new(commands: &str) -> Result<CommandPipeline, String>{
        let commands = CommandLine::from(commands)?;

        let mut commands = &commands[..];
        let mut execution_policy = ExecutionPolicy::Foreground; 
        if let [actual_commands @ .., Token::Arg(last)] = commands && last == "&"{
            commands = actual_commands;
            execution_policy = ExecutionPolicy::Background;
        }
        let raw = CommandLine(commands.to_vec());

        let mut commands = commands
            .split(|t| matches!(t, Token::Pipe))
            .map(|cmd| Command::new(&CommandLine(cmd.to_vec())))
            .collect::<Result<Vec<_>, _>>()?;

        let idx = commands
            .iter()
            .rposition(|cmd| !matches!(cmd.r#type, command::Kind::Executable{..}));

        let first_command = match idx{
            Some(idx) => {
                let rest = commands.drain(0..=idx);

                Some(rest.last().unwrap())
            },
            None => None
        };

        let processes = commands.iter().map(|cmd| {
            let command::Kind::Executable { ref name, ref args, .. } = cmd.r#type else {
                unreachable!();
            };

            Program::new(name).args(args)
        }).collect::<Vec<_>>();

        let redirection = if !commands.is_empty() {
            commands.pop().unwrap().redirection.clone()
        } else {
            first_command.as_ref().unwrap().redirection.clone()
        };

        Ok(CommandPipeline{
            raw,
            first_command,
            processes,
            execution_policy,
            redirection
        })
    }
}

impl Shell{
    fn try_execute_cmd_pipeline(&mut self, mut pipeline: CommandPipeline) -> Result<ExecutionStatus, String>{
        let mut processes = pipeline.processes;

        let Some((last, all_but_last)) = processes.split_last_mut() else {
            pipeline.first_command.as_mut().unwrap().execution_policy = pipeline.execution_policy;
            return Ok(self.execute_command(pipeline.first_command.unwrap()));
        };

        if let Some(redirection) = pipeline.redirection{
            let should_append = matches!(redirection.r#type, RedirectionType::Append);

            let filename = redirection.target.filepath();
            let file = OpenOptions::new()
                .write(true)
                .append(should_append)
                .truncate(!should_append)
                .create(true)
                .open(filename)
                .map_err(|err| format!("{err}"))?;

            match redirection.target{
                RedirectionTarget::Stdout(_) => last.command.stdout(file),
                RedirectionTarget::Stderr(_) => last.command.stderr(file)
            };
        }

        for proc in &mut *all_but_last{
            proc.command.stdout(Stdio::piped());
        }

        let first_proc = match all_but_last{
            [first, ..] => first,
            _ => last
        };

        if pipeline.first_command.is_some(){
            first_proc.command.stdin(Stdio::piped());
        }

        let mut current_child = match first_proc.spawn(){
            Ok(child) => child,
            Err(err) => return Err(format!("{err}"))
        };

        if let Some(command) = pipeline.first_command{
            let first_stdin = self.command_output(&command).stdout;

            let mut stdin = current_child.stdin.take().ok_or(String::from("stdin error"))?;
            writeln!(stdin, "{}", first_stdin).map_err(|err| format!("{err}"))?;
        }

        for proc in &mut processes[1..]{
            proc.command
                .stdin(current_child.stdout.take().ok_or("stdout error")?);
            
            current_child = proc.spawn().map_err(|err| format!("{err}"))?;
        }

        match pipeline.execution_policy{
            ExecutionPolicy::Foreground => {
                current_child.wait().map_err(|err| format!("{err}"))?;
            },
            ExecutionPolicy::Background => {
                let out = self.start_job(&pipeline.raw, current_child);
                print!("{out}");
            }
        }

        Ok(ExecutionStatus::Continue)
    }

    pub fn execute_cmd_pipeline(&mut self, pipeline: CommandPipeline) -> ExecutionStatus{
        match self.try_execute_cmd_pipeline(pipeline){
            Ok(status) => status,
            Err(err) => {
                println!("{err}");
                ExecutionStatus::Continue
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_first_command_existence(){
        let pipeline = CommandPipeline::new("ls | wc").unwrap();

        assert!(pipeline.first_command.is_none());
    }

    #[test]
    fn test_proc_count(){
        let pipeline = CommandPipeline::new("ls | wc").unwrap();

        assert!(pipeline.processes.len() == 2);
    }
}
