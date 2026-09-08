use crate::command::{Command, CommandArgs, ExecutionStatus, Output, OutputType, stringify_args};
use crate::command;
use crate::lexer::{ tokens_to_string, Token };
use crate::shell::{Shell, Job};
use crate::program::Program;
use std::env;
use std::path::Path;

impl Shell{
    pub fn execute_command(&mut self, command: Command) -> ExecutionStatus{
        if let command::Kind::Exit = command.r#type{
            return ExecutionStatus::Exit;
        } 

        let (output_type, cmd_args) = match &command.args[..]{
            [rest @ .., Token::Arg(val)] if val == "&" => {
                (OutputType::Both, rest)
            },
            _ => (match command.redirection{
                None => OutputType::Display,
                Some(_) => OutputType::Redirect,
            }, &command.args[..])
        };

        let result = match command.r#type{
            command::Kind::Exit => unreachable!(),
            command::Kind::Echo => self.execute_echo(cmd_args),
            command::Kind::Type => self.execute_type(cmd_args),
            command::Kind::Pwd => self.execute_pwd(),
            command::Kind::Cd => self.execute_cd(cmd_args),
            command::Kind::Complete => self.execute_complete(cmd_args),
            command::Kind::Jobs => self.execute_jobs(),
            command::Kind::Executable{ name, .. } => self.execute_program(&name, cmd_args, output_type),
        };

        let Some(redirection) = command.redirection else{
            print!("{result}");
            return ExecutionStatus::Continue;
        };

        result.redirect(redirection);

        ExecutionStatus::Continue
    }

    fn execute_jobs(&self) -> Output{
        let total_jobs = self.jobs.len();

        let output = self.jobs.iter()
            .enumerate()
            .map(|(idx, job)| {
                let symbol = match idx{
                    _ if idx + 1 == total_jobs => '+',
                    _ if idx + 2 == total_jobs => '-',
                    _ => ' '
                };
                format!("[{}]{}  Running{:17}{}\n", idx + 1, symbol, "", job.command)
            })
            .collect::<Vec<_>>()
            .join("");
            

        Output::stdout(output.trim_end())
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

    fn start_job(&mut self, executable: &str, args: &CommandArgs, pid: u32) -> Output{
        let command = format!("{} {}", executable, stringify_args(args));

        self.jobs.push(Job{ pid, command });

        Output::stdout(&format!("[{}] {}", self.jobs.len(), pid))
    }

    pub fn execute_program(&mut self, executable: &str, args: &CommandArgs, output_type: OutputType) -> Output{
        let mut program = Program::new(executable)
            .args(args);
        
        match output_type{
            OutputType::Both => {
                match program.spawn(){
                    Ok(child) => self.start_job(executable, args, child.id()),
                    Err(err) => {
                        eprintln!("{err}");
                        Output::empty()
                    }
                }
            }
            OutputType::Redirect => program.output(),
            OutputType::Display => {
                program.spawn_and_wait();
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
