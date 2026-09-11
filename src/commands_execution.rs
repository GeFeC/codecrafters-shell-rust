use crate::command::{Command, CommandArgs, ExecutionPolicy, ExecutionStatus, Output, OutputType};
use crate::command;
use crate::lexer::{ tokens_to_string, Token };
use crate::shell::{Shell};
use crate::jobs::JobState;
use crate::program::Program;
use std::env;
use std::path::Path;
use std::process::{Child, Stdio};
use command::BuiltinCommand;

impl Shell{
    pub fn command_output(&mut self, command: &Command) -> Output{
        let output_type = match command.execution_policy{
            ExecutionPolicy::Background => OutputType::Background,
            ExecutionPolicy::Foreground => match command.redirection{
                None => OutputType::Display,
                Some(_) => OutputType::Redirect,
            }
        };

        let cmd_args = &command.args[..];
        match &command.r#type{
            command::Kind::Builtin(command) => match command{
                BuiltinCommand::Exit => Output::empty(),
                BuiltinCommand::Echo => self.execute_echo(cmd_args),
                BuiltinCommand::Type => self.execute_type(cmd_args),
                BuiltinCommand::Pwd => self.execute_pwd(),
                BuiltinCommand::Cd => self.execute_cd(cmd_args),
                BuiltinCommand::Complete => self.execute_complete(cmd_args),
                BuiltinCommand::Jobs => self.execute_jobs(),
            }
            command::Kind::Executable{ name, .. } => self.execute_program(&[&[Token::Arg(name.clone())], cmd_args].concat(), output_type),
        }
    }

    pub fn execute_command(&mut self, command: Command) -> ExecutionStatus{
        if let command::Kind::Builtin(BuiltinCommand::Exit) = command.r#type{
            return ExecutionStatus::Exit;
        } 

        let result = self.command_output(&command);

        match command.redirection{
            Some(redirection) => {
                result.redirect(&redirection);
            },
            None => {
                print!("{result}");
            }
        }

        self.update_jobs();
        let jobs = self.display_jobs(|(_, job)| matches!(job.state, JobState::Done));
        if !jobs.is_empty(){
            println!("{}", jobs);
        }
        self.remove_finished_jobs();

        ExecutionStatus::Continue
    }

    fn execute_jobs(&mut self) -> Output{
        self.update_jobs();
        let output = Output::stdout(&self.display_jobs(|_| true));
        self.remove_finished_jobs();
        output
    }

    fn execute_echo(&self, args: &CommandArgs) -> Output{
        Output::stdout(&tokens_to_string(args))
    }

    fn execute_pwd(&self) -> Output{
        Output::stdout(&format!("{}", env::current_dir().unwrap().display()))
    }

    fn execute_cd(&self, args: &CommandArgs) -> Output{
        let home_dir = env::home_dir();

        let path = tokens_to_string(args);
        let path = match path.as_str(){
            "~" | "" => home_dir.as_deref().and_then(Path::to_str).unwrap_or("~").to_owned(),
            _ => path
        };

        if env::set_current_dir(&path).is_err(){
            return Output::stderr(&format!("cd: {}: No such file or directory", path));
        }

        Output::empty()
    }

    fn execute_type(&self, checked_cmd: &CommandArgs) -> Output{
        if checked_cmd.len() != 1{
            return Output::stderr("Usage: type <command name without args>");
        }

        let cmd_name = checked_cmd.first().unwrap().to_string();
        let command = Command::from_str(&cmd_name);

        if command.is_err(){
            return Output::stderr(&format!("{}: not found", cmd_name));
        }

        let command = command.unwrap();

        if let command::Kind::Executable { path, .. } = command.r#type{
            return Output::stdout(&format!("{} is {}", cmd_name, path));
        }

        Output::stdout(&format!("{} is a shell builtin", cmd_name))
    }

    fn execute_complete(&mut self, args: &CommandArgs) -> Output{
        let completions_paths = &mut self.editor.helper_mut().unwrap().completions_paths;

        match args{
            [Token::Arg(flag), Token::Arg(path), Token::Arg(cmd), ..] if flag == "-C" => {
                completions_paths.insert(cmd.clone(), path.clone());
                Output::empty()
            },

            [Token::Arg(flag), Token::Arg(cmd), ..] if flag == "-p" => {
                match completions_paths.get(cmd){
                    Some(path) => Output::stdout(&format!("complete -C '{path}' {cmd}")),
                    None => Output::stdout(&format!("complete: {}: no completion specification", cmd))
                }
            },

            [Token::Arg(flag), Token::Arg(cmd), ..] if flag == "-r" => {
                completions_paths.remove(cmd);
                Output::empty()
            },
            _ => Output::stderr("Usage: complete <'-p'/'-C'/'-r'> <command>")
        }
    }

    fn start_job(&mut self, command: &CommandArgs, proc: Child) -> Output{
        let command = format!("{}", tokens_to_string(command));

        let pid = proc.id();
        let job = self.new_job(proc, command);
        let job_idx = job.idx;
        self.jobs.push(job);

        Output::stdout(&format!("[{}] {}", job_idx, pid))
    }

    pub fn execute_program(&mut self, args: &CommandArgs, output_type: OutputType) -> Output{
        let processes = args.split(|t| matches!(t, Token::Pipe)).map(|proc| {
            let [Token::Arg(cmd_name), args @ ..] = proc else {
                return None;
            };

            Some(Program::new(cmd_name).args(args))
        }).collect::<Option<Vec<_>>>();

        let Some(mut processes) = processes else{
            return Output::stderr("Invalid command name");
        };

        for i in 0..(processes.len() - 1){
            let (proc, next_proc) = processes.split_at_mut(i + 1);
            let proc = &mut proc[i];
            let next_proc = &mut next_proc[0];

            proc.command.stdout(Stdio::piped());
            match &mut proc.spawn(){
                Ok(proc) => {
                    next_proc.command.stdin(proc.stdout.take().unwrap())
                },
                Err(err) => {
                    return Output::stderr(&format!("{err}"));
                }
            };
        }

        let last_proc = processes.last_mut().unwrap();

        match output_type{
            OutputType::Background => {
                match last_proc.spawn(){
                    Ok(child) => {
                        self.start_job(args, child)
                    },
                    Err(err) => {
                        eprintln!("{err}");
                        Output::empty()
                    }
                }
            }
            OutputType::Redirect => last_proc.output(),
            OutputType::Display => {
                last_proc.spawn_and_wait();
                Output::empty()
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::lexer::Token;

    fn test_type_helper(cmd: &str) -> Output{
        let shell = Shell::new().unwrap();
        shell.execute_type(&[Token::Arg(cmd.to_owned())])
    }

    #[test]
    fn test_type(){
        let output = test_type_helper("cat");
        assert_ne!(output.stdout, "cat is a shell builtin");

        let output = test_type_helper("echo");
        assert_eq!(output.stdout, "echo is a shell builtin");
    }
}
