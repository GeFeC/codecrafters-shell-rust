use crate::commands_execution;
use crate::env;
use crate::lexer;
use std::fs::File;
use std::io::Write;

pub type CommandArgs = Vec<String>;

pub struct Command{
    r#type: Type,
    redirection: Redirection
}

pub struct Redirection(Option<String>);

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

pub enum Type{
    Exit,
    Pwd,
    Echo(CommandArgs),
    #[allow(clippy::enum_variant_names)]
    Type(CommandArgs),
    Cd(CommandArgs),
    Executable{ name: String, args: CommandArgs }
}

impl Type{
    pub fn from(command: &str, command_args: CommandArgs) -> Result<Type, String>{
        match command{
            "exit" => Ok(Type::Exit),
            "echo" => Ok(Type::Echo(command_args)),
            "type" => Ok(Type::Type(command_args)),
            "pwd" => Ok(Type::Pwd),
            "cd" => Ok(Type::Cd(command_args)),
            _ if env::find_in_path(command).is_some() => Ok(Type::Executable{
                name: command.to_owned(),
                args: command_args
            }),
            _ => Err(format!("{}: command not found", command))
        }
    }
}

impl Command{
    pub fn from_str(command: &str) -> Result<Command, String>{
        let command = lexer::split_command(command);
        if command.is_none(){
            return Err(String::from("Missing closing quote"));
        }

        let command = command.unwrap();
        let command_name = command.first().unwrap().as_str();
        let command_args = command[1..].to_owned();

        let (command_args, redirection) = match command_args.iter().position(|e| e == ">"){
            Some(idx) => {
                let (new_cmd_args, target_file) = command_args.split_at(idx);
                let target_file = &target_file[1..];

                let redirection = if target_file.is_empty(){
                    None
                }
                else{
                    Some(target_file.join(" "))
                };

                (new_cmd_args, redirection)
            },
            None => (&command_args[..], None)
            
        };

        let command_type = Type::from(command_name, command_args.to_vec())?;

        Ok(Command{
            r#type: command_type,
            redirection: Redirection(redirection)
        })
    }

    pub fn execute(self) -> ExecutionStatus{
        if let Type::Exit = self.r#type{
            return ExecutionStatus::Exit;
        }

        let Redirection(redirection) = self.redirection;

        let output_type = match redirection{
            Some(_) => OutputType::Redirect,
            None => OutputType::Display
        };

        let result = match self.r#type{
            Type::Echo(args) => commands_execution::execute_echo(&args),
            Type::Type(args) => commands_execution::execute_type(&args),
            Type::Pwd => commands_execution::execute_pwd(),
            Type::Cd(args) => commands_execution::execute_cd(&args),
            Type::Executable{ name, args } => commands_execution::execute_process(&name, &args, output_type),
            _ => Output::empty()
        };

        let Output{ stdout, stderr } = result;

        let stdout = stdout.trim();

        match redirection{
            Some(file) => {
                let file = File::create(file).and_then(|mut f| {
                    f.write_all(stdout.as_bytes())
                });

                if let Err(err) = file{
                    println!("Unable to write to file: {}", err);
                }
            },
            None if !stdout.is_empty() => {
                println!("{stdout}");
            },
            None => {}
        }

        if !stderr.is_empty(){
            println!("{stderr}");
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
mod test{
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_output_redirection(){
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fruits.txt");
        let expected_content = "apple banana orange";

        let command = format!(r#"echo "{}" > {}"#, expected_content, file_path.display());
        Command::from_str(&command).unwrap().execute();

        assert!(file_path.exists(), "The file wasn't created");
        let content = fs::read_to_string(&file_path).unwrap();

        assert_eq!(content, expected_content);
    }
}
