use std::process::Stdio;
use std::string;
use std::thread::current;

use crate::command::{ExecutionStatus, Output};

use std::io::Write;
use crate::lexer::{CommandLine, Token};
use crate::command::{Command, ExecutionPolicy};
use crate::command;
use crate::program::Program;
use crate::shell::Shell;

pub struct CommandPipeline{
    pub first_command: Command,
    pub processes: Vec<Program>,
    pub execution_policy: ExecutionPolicy
}

impl CommandPipeline{
    pub fn from_str(commands: &str) -> Result<CommandPipeline, String>{
        Self::new(&CommandLine::from(commands)?)
    }

    pub fn new(commands: &CommandLine) -> Result<CommandPipeline, String>{
        let mut commands = &commands[..];
        let mut execution_policy = ExecutionPolicy::Foreground; 
        if let [actual_commands @ .., Token::Arg(last)] = commands && last == "&"{
            commands = actual_commands;
            execution_policy = ExecutionPolicy::Background;
        }

        let mut commands = commands
            .split(|t| matches!(t, Token::Pipe))
            .map(|cmd| Command::new(&CommandLine(cmd.to_vec())))
            .collect::<Result<Vec<_>, _>>()?;

        let idx = commands
            .iter()
            .rposition(|cmd| !matches!(cmd.r#type, command::Kind::Executable{..}))
            .unwrap_or(0);

        let rest = commands.drain(0..=idx);
        let first_command = rest.last().unwrap();

        let processes = commands.iter().map(|cmd| {
            let command::Kind::Executable { ref name, ref args, .. } = cmd.r#type else {
                unreachable!();
            };

            Program::new(name).args(args)
        }).collect::<Vec<_>>();

        Ok(CommandPipeline{
            first_command,
            processes,
            execution_policy
        })
    }
}

impl Shell{
    fn try_execute_cmd_pipeline(&mut self, mut pipeline: CommandPipeline) -> Result<ExecutionStatus, String>{
        let mut processes = pipeline.processes;

        let Some((last, rest)) = processes.split_last_mut() else {
            pipeline.first_command.execution_policy = pipeline.execution_policy;
            return Ok(self.execute_command(pipeline.first_command));
        };

        let first_output = self.command_output(&pipeline.first_command);

        for proc in &mut *rest{
            proc.command.stdout(Stdio::piped());
        }

        let first_proc = match rest{
            [first, ..] => first,
            _ => last
        };
        first_proc.command.stdin(Stdio::piped());
        let mut current_child = match first_proc.spawn(){
            Ok(child) => child,
            Err(err) => return Err(format!("{err}"))
        };

        let stdin = current_child.stdin.as_mut().ok_or(String::from("stdin error"))?;
        write!(stdin, "{}", first_output).map_err(|err| format!("{err}"))?;

        for proc in &mut processes[1..]{
            proc.command
                .stdin(current_child.stdout.take().ok_or("stdout error")?);
            
            current_child = proc.spawn().map_err(|err| format!("{err}"))?;
        }

        current_child.wait().map_err(|err| format!("{err}"))?;

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
