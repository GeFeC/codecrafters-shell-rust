use crate::env;
use crate::lexer::{CommandLine, Token};
use crate::output_redirection;
use crate::output_redirection::{Redirection};

use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use rustyline::history::History;
use strum_macros::{EnumString, VariantNames};
use crate::shell::{Shell};
use crate::jobs::JobState;
use crate::program::Program;
use std::path::Path;

pub enum ExecutionPolicy{
    Foreground,
    Background
}

pub struct Command{
    pub r#type: Kind,
    pub redirection: Option<Redirection>,
    pub execution_policy: ExecutionPolicy,
}

pub struct Output{
    pub stdout: String,
    pub stderr: String
}

impl Output{
    pub fn empty() -> Output{
        Output{
            stdout: String::new(),
            stderr: String::new()
        }
    }

    pub fn stdout(str: &str) -> Output{
        Output{
            stdout: str.to_owned(),
            stderr: String::new()
        }
    }

    pub fn stderr(str: &str) -> Output{
        Output{
            stderr: str.to_owned(),
            stdout: String::new()
        }
    }
}

impl std::fmt::Display for Output{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error>{
        let stdout = match &self.stdout{
            s if s.is_empty() => "",
            s => &format!("{s}\n")
        };

        let stderr = match &self.stderr{
            s if s.is_empty() => "",
            s => &format!("{s}\n")
        };

        write!(fmt, "{}{}", stdout, stderr)
    }
}

pub enum OutputType{
    Display,
    Redirect,
    Background //For ExecutionPolicy::Background
}

#[derive(PartialEq, EnumString, VariantNames)]
pub enum BuiltinCommand{
    #[strum(to_string = "exit")]
    Exit,
    #[strum(to_string = "pwd")]
    Pwd,
    #[strum(to_string = "echo")]
    Echo,
    #[strum(to_string = "type")]
    Type,
    #[strum(to_string = "cd")]
    Cd,
    #[strum(to_string = "complete")]
    Complete,
    #[strum(to_string = "jobs")]
    Jobs,
    #[strum(to_string = "history")]
    History,
}

pub enum Kind{
    Builtin{ name: BuiltinCommand, args: CommandLine },
    Executable{ name: String, full_path: String, args: CommandLine },
}

impl Command{
    pub fn from_str(command: &str) -> Result<Command, String>{
        Self::new(&CommandLine::from(command)?)
    }

    pub fn new(command: &CommandLine) -> Result<Command, String>{
        let [Token::Arg(command_name), command_args @ ..] = &command[..] else {
            return Err(String::from("Invalid command name"));
        };

        let mut command_args = command_args;
        let mut execution_policy = ExecutionPolicy::Foreground; 
        if let [actual_command_args @ .., Token::Arg(last)] = command_args && last == "&"{
            command_args = actual_command_args; 
            execution_policy = ExecutionPolicy::Background;
        }

        let execution_policy = execution_policy;

        let (redirection, command_args) = output_redirection::get_redirection_and_new_command_args(
            command_args.to_vec()
        );

        let command_type = if let Ok(command) = BuiltinCommand::from_str(command_name){
            Ok(Kind::Builtin{ name: command, args: CommandLine(command_args) })
        } 
        else if let Some(path) = env::find_in_path(command_name){
            Ok(Kind::Executable {
                name: command_name.to_owned(),
                full_path: path,
                args: CommandLine(command_args)
            })
        }
        else {
            Err(format!("{}: command not found", command_name))
        };
        
        Ok(Command{
            r#type: command_type?,
            redirection,
            execution_policy
        })
    }
}


impl Shell{
    fn execute_command_helper(&mut self, command: &Command, output_type: OutputType) -> Output{
        match &command.r#type{
            Kind::Builtin{ name, args } => match name{
                BuiltinCommand::Exit => Output::empty(),
                BuiltinCommand::Echo => self.execute_echo(args),
                BuiltinCommand::Type => self.execute_type(args),
                BuiltinCommand::Pwd => self.execute_pwd(),
                BuiltinCommand::Cd => self.execute_cd(args),
                BuiltinCommand::Complete => self.execute_complete(args),
                BuiltinCommand::Jobs => self.execute_jobs(),
                BuiltinCommand::History => self.execute_history(args)
            }
            Kind::Executable{ name, args, .. } => self.execute_program(name, args, output_type),
        }
    }

    pub fn command_output(&mut self, command: &Command) -> Output{
        self.execute_command_helper(command, OutputType::Redirect)
    }

    pub fn execute_command(&mut self, command: Command) -> ExecutionStatus{
        if let Kind::Builtin{ name: BuiltinCommand::Exit, .. } = command.r#type{
            return ExecutionStatus::Exit;
        } 

        let output_type = match command.execution_policy{
            ExecutionPolicy::Background => OutputType::Background,
            ExecutionPolicy::Foreground => match command.redirection{
                None => OutputType::Display,
                Some(_) => OutputType::Redirect,
            }
        };

        let result = self.execute_command_helper(&command, output_type);

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

    fn execute_history(&mut self, args: &CommandLine) -> Output{
        if let [Token::Arg(flag), Token::Arg(path)] = &args[..]{
            match flag.as_str(){
                "-r" => {
                    if self.editor.load_history(path).is_err(){
                        return Output::stderr(&format!("{path}: file not found"));
                    }
                    return Output::empty();
                },
                flag @ ("-w" | "-a") => {
                    let should_append = flag == "-a";

                    let saving_status = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .append(should_append)
                        .truncate(!should_append)
                        .open(path)
                        .and_then(|mut f|{
                            for entry in self.editor.history(){
                                writeln!(f, "{entry}")?;
                            }

                            Ok(())
                        });


                    if let Err(err) = saving_status{
                        return Output::stderr(&format!("{err}"));
                    }

                    if should_append{
                        self.editor.clear_history().unwrap_or(());
                    }

                    return Output::empty();
                },
                _ => return Output::stderr(&format!("history: {flag}: invalid option"))
            }
        }

        let limited = !args.is_empty();
        let last_n = match &args[..]{
            [] => 0,
            [Token::Arg(n)] => match n.parse::<usize>(){
                Ok(n) => n,
                Err(_) => return Output::stderr(&format!("history: {n}: numeric argument required"))
            },
            _ => return Output::stderr("history: too many arguments")
        };

        let history_size = self.editor.history().len();
        let output = self.editor.history()
            .iter()
            .enumerate()
            .filter(|(idx, _)| !limited || idx + last_n >= history_size)
            .map(|(idx, entry)| format!("    {}  {entry}", idx + 1))
            .collect::<Vec<_>>()
            .join("\n");

        Output::stdout(&output)
    }

    fn execute_echo(&self, args: &CommandLine) -> Output{
        Output::stdout(&args.to_string())
    }

    fn execute_pwd(&self) -> Output{
        Output::stdout(&format!("{}", std::env::current_dir().unwrap().display()))
    }

    fn execute_cd(&self, args: &CommandLine) -> Output{
        let home_dir = std::env::home_dir();

        let path = args.to_string();
        let path = match path.as_str(){
            "~" | "" => home_dir.as_deref().and_then(Path::to_str).unwrap_or("~").to_owned(),
            _ => path
        };

        if std::env::set_current_dir(&path).is_err(){
            return Output::stderr(&format!("cd: {}: No such file or directory", path));
        }

        Output::empty()
    }

    fn execute_type(&self, checked_cmd: &CommandLine) -> Output{
        if checked_cmd.len() != 1{
            return Output::stderr("Usage: type <command name without args>");
        }

        let cmd_name = checked_cmd.first().unwrap().to_string();
        let command = Command::from_str(&cmd_name);

        if command.is_err(){
            return Output::stderr(&format!("{}: not found", cmd_name));
        }

        let command = command.unwrap();

        if let Kind::Executable { full_path, .. } = command.r#type{
            return Output::stdout(&format!("{} is {}", cmd_name, full_path));
        }

        Output::stdout(&format!("{} is a shell builtin", cmd_name))
    }

    fn execute_complete(&mut self, args: &CommandLine) -> Output{
        let completions_paths = &mut self.editor.helper_mut().unwrap().completions_paths;

        match &args[..]{
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

    pub fn execute_program(&mut self, program_name: &str, args: &CommandLine, output_type: OutputType) -> Output{
        let mut program = Program::new(program_name).args(args);

        match output_type{
            OutputType::Background => {
                match program.spawn(){
                    Ok(child) => {
                        self.start_job(args, child)
                    },
                    Err(err) => {
                        eprintln!("{err}");
                        Output::empty()
                    }
                }
            }
            OutputType::Redirect => program.output(),
            OutputType::Display => {
                if let Err(err) = program.spawn_and_wait(){
                    return Output::stderr(&err);
                }
                Output::empty()
            }
        }
    }
}

pub enum ExecutionStatus{
    Exit,
    Continue
}

impl ExecutionStatus{
    pub fn should_exit(self) -> bool{
        match self{
            ExecutionStatus::Exit => true,
            ExecutionStatus::Continue => false
        }
    }
}


#[cfg(test)]
mod tests{
    use crate::shell::Shell;

    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn execute(cmd: &str){
        let mut shell = Shell::new().unwrap();
        let command = Command::from_str(cmd).unwrap();

        shell.execute_command(command);
    }

    #[test]
    fn test_output_redirection(){
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fruits.txt");
        let expected_content = "apple banana orange";

        execute(&format!(r#"echo "{}" > {}"#, expected_content, file_path.display()));

        assert!(file_path.exists(), "The file wasn't created");
        let content = fs::read_to_string(&file_path).unwrap();

        assert_eq!(content, format!("{}\n", expected_content));
    }

    #[test]
    fn test_stderr_redirection(){
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fruits.txt");

        execute(&format!(r#"echo "apple banana orange" 2> {}"#, file_path.display()));

        assert!(file_path.exists(), "The file wasn't created");
        let content = fs::read_to_string(&file_path).unwrap();

        assert_eq!(content, "");
    }

    fn test_type_helper(cmd: &str) -> Output{
        let shell = Shell::new().unwrap();
        shell.execute_type(&CommandLine(vec![Token::Arg(cmd.to_owned())]))
    }

    #[test]
    fn test_type(){
        let output = test_type_helper("cat");
        assert_ne!(output.stdout, "cat is a shell builtin");

        let output = test_type_helper("echo");
        assert_eq!(output.stdout, "echo is a shell builtin");
    }
}
