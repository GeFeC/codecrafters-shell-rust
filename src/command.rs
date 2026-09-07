use crate::commands_execution;
use crate::env;
use crate::lexer;
use std::str::FromStr;
use strum_macros::EnumString;
use strum_macros::VariantNames;
use std::fs::{ OpenOptions };
use std::io::Write;

pub type CommandArgs = [lexer::Token];
pub type CommandArgsStorage = Vec<lexer::Token>;

pub struct Command{
    pub r#type: Kind,
    args: CommandArgsStorage,
    redirection: Option<Redirection>
}

pub struct Redirection{
    target: RedirectionTarget,
    r#type: RedirectionType
}

pub enum RedirectionTarget{
    Stdout(String),
    Stderr(String)
}

pub enum RedirectionType{
    Overwrite,
    Append
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
    fn get_redirection_and_new_command_args(command_args: Vec<lexer::Token>) -> (Option<Redirection>, Vec<lexer::Token>){
        let symbol_pos = command_args.iter().position(|e| e.is_redirection());

        let (symbol, command_args, target_file) = match symbol_pos{
            Some(idx) => {
                let symbol = &command_args[idx];
                let (command_args, target_file) = command_args.split_at(idx);
                let target_file = &target_file[1..];

                (Some(symbol), command_args.to_vec(), target_file.to_vec())
            },
            None => (None, command_args, Vec::<Token>::new())
        };

        if symbol.is_none(){
            return (None, command_args.to_vec());
        }

        let symbol = symbol.unwrap();
        use lexer::Token;

        let Token::Arg(target_file) = target_file.into_iter().nth(0).unwrap_or(Token::Stdout) else{ 
            panic!("Wrong redirection file name not handled!");
        };

        let target = match symbol{
            Token::Stdout | Token::StdoutAppend => RedirectionTarget::Stdout(target_file),
            Token::Stderr | Token::StderrAppend => RedirectionTarget::Stderr(target_file),
            _ => unreachable!()
        };
        
        let r#type = match symbol{
            Token::Stdout | Token::Stderr => RedirectionType::Overwrite,
            Token::StdoutAppend | Token::StderrAppend => RedirectionType::Append,
            _ => unreachable!()
        };

        (Some(Redirection{ target, r#type }), command_args)
    }

    pub fn from_str(command: &str) -> Result<Command, String>{
        let command = lexer::split_command(command)?;

        if let lexer::Token::Arg(command_name) = command.first().unwrap(){
            let command_args = &command[1..];

            let (redirection, command_args) = Command::get_redirection_and_new_command_args(command_args.to_vec());
            
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

    pub fn execute(self) -> ExecutionStatus{
        if let Kind::Exit = self.r#type{
            return ExecutionStatus::Exit;
        } 

        let output_type = match self.redirection{
            None => OutputType::Display,
            Some(_) => OutputType::Redirect,
        };

        let result = match self.r#type{
            Kind::Exit => unreachable!(),
            Kind::Echo => commands_execution::execute_echo(&self.args),
            Kind::Type => commands_execution::execute_type(&self.args),
            Kind::Pwd => commands_execution::execute_pwd(),
            Kind::Cd => commands_execution::execute_cd(&self.args),
            Kind::Complete => commands_execution::execute_complete(&self.args),
            Kind::Executable{ name, .. } => commands_execution::execute_process(&name, &self.args, output_type),
        };

        let Output{ stdout, stderr } = result;

        let Some(redirection) = self.redirection else{
            if !stdout.is_empty(){
                println!("{stdout}");
            }
            if !stderr.is_empty(){
                println!("{stderr}");
            }
            return ExecutionStatus::Continue;
        };

        if !matches!(redirection.target, RedirectionTarget::Stdout(_)) && !stdout.is_empty(){
            println!("{stdout}");
        }

        if !matches!(redirection.target, RedirectionTarget::Stderr(_)) && !stderr.is_empty(){
            println!("{stderr}");
        }

        let (out, file) = match redirection.target{
            RedirectionTarget::Stdout(file) => (stdout, file),
            RedirectionTarget::Stderr(file) => (stderr, file),
        };

        let should_append = matches!(redirection.r#type, RedirectionType::Append);
        let file = OpenOptions::new()
            .write(true)
            .append(should_append)
            .truncate(!should_append)
            .create(true)
            .open(file)
            .and_then(|mut f|{
                write!(f, "{out}")?;
                if !out.is_empty() && !out.ends_with("\n"){
                    return writeln!(f);
                }

                Ok(())
            });

        if let Err(err) = file{
            println!("{}", err);
        }

        ExecutionStatus::Continue
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
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn execute(cmd: &str){
        Command::from_str(cmd).unwrap().execute();
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
