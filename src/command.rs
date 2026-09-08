use crate::env;
use crate::lexer;
use crate::output_redirection;
use std::str::FromStr;
use strum_macros::EnumString;
use strum_macros::VariantNames;
use crate::output_redirection::{Redirection};

pub type CommandArgs = [lexer::Token];
pub type CommandArgsStorage = Vec<lexer::Token>;

pub struct Command{
    pub r#type: Kind,
    pub args: CommandArgsStorage,
    pub redirection: Option<Redirection>
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

pub enum OutputType{
    Display,
    Redirect
}

#[derive(PartialEq, EnumString, VariantNames)]
pub enum Kind{
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
    #[strum(to_string = "")]
    Executable{ name: String, path: String },
}

impl Command{
    pub fn from_str(command: &str) -> Result<Command, String>{
        let command = lexer::split_command(command)?;

        if let lexer::Token::Arg(command_name) = command.first().unwrap(){
            let command_args = &command[1..];

            let (redirection, command_args) = output_redirection::get_redirection_and_new_command_args(
                command_args.to_vec()
            );
            
            let command_type = match Kind::from_str(command_name){
                Ok(command) => Ok(command),
                Err(_) if let Some(path) = env::find_in_path(command_name) => Ok(Kind::Executable{
                    name: command_name.to_owned(),
                    path
                }),
                Err(_) => Err(format!("{}: command not found", command_name))
            };

            return Ok(Command{
                r#type: command_type?,
                args: command_args,
                redirection
            });
        }

        Err(String::from("Invalid command name"))
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
}
