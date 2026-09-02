use crate::commands_execution;
use crate::env;
use crate::lexer;
use std::fs::{ OpenOptions };
use std::io::Write;

pub type CommandArgs = Vec<String>;

pub struct Command{
    pub r#type: Type,
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

pub enum Type{
    Exit,
    Pwd,
    Echo(CommandArgs),
    #[allow(clippy::enum_variant_names)]
    Type(CommandArgs),
    Cd(CommandArgs),
    Executable{ name: String, args: CommandArgs, path: String }
}

impl Type{
    pub fn from(command: &str, command_args: CommandArgs) -> Result<Type, String>{
        match command{
            "exit" => Ok(Type::Exit),
            "echo" => Ok(Type::Echo(command_args)),
            "type" => Ok(Type::Type(command_args)),
            "pwd" => Ok(Type::Pwd),
            "cd" => Ok(Type::Cd(command_args)),
            _ if let Some(path) = env::find_in_path(command) => Ok(Type::Executable{
                name: command.to_owned(),
                args: command_args,
                path
            }),
            _ => Err(format!("{}: command not found", command))
        }
    }
}

impl Command{
    fn get_redirection_and_new_command_args(command_args: &[String]) -> (Option<Redirection>, &[String]){
        const REDIRECTION_SYMBOLS: [&str; 4] = [">", "2>", ">>", "2>>"];

        let symbol_pos = command_args.iter().position(|e| REDIRECTION_SYMBOLS.contains(&e.as_str()));

        let (symbol, command_args, target_file) = match symbol_pos{
            Some(idx) => {
                let symbol = command_args[idx].as_str();
                let (command_args, target_file) = command_args.split_at(idx);
                let target_file = target_file[1..].join(" ");

                (symbol, command_args, target_file)
            },
            None => ("", command_args, String::new())
        };

        if !REDIRECTION_SYMBOLS.contains(&symbol){
            return (None, command_args);
        }

        let target = match symbol{
            ">" | ">>" => RedirectionTarget::Stdout(target_file),
            "2>" | "2>>" => RedirectionTarget::Stderr(target_file),
            _ => unreachable!()
        };
        
        let r#type = match symbol{
            ">" | "2>" => RedirectionType::Overwrite,
            ">>" | "2>>" => RedirectionType::Append,
            _ => unreachable!()
        };

        (Some(Redirection{ target, r#type }), command_args)
    }

    pub fn from_str(command: &str) -> Result<Command, String>{
        let command = lexer::split_command(command);
        if command.is_none(){
            return Err(String::from("Missing closing quote"));
        }

        let command = command.unwrap();
        let command_name = command.first().unwrap().as_str();
        let command_args = command[1..].to_owned();

        let (redirection, command_args) = Command::get_redirection_and_new_command_args(&command_args);
        
        let command_type = Type::from(command_name, command_args.to_vec())?;

        Ok(Command{
            r#type: command_type,
            redirection
        })
    }

    pub fn execute(self) -> ExecutionStatus{
        if let Type::Exit = self.r#type{
            return ExecutionStatus::Exit;
        } 

        let output_type = match self.redirection{
            None => OutputType::Display,
            Some(_) => OutputType::Redirect,
        };

        let result = match self.r#type{
            Type::Echo(args) => commands_execution::execute_echo(&args),
            Type::Type(args) => commands_execution::execute_type(&args),
            Type::Pwd => commands_execution::execute_pwd(),
            Type::Cd(args) => commands_execution::execute_cd(&args),
            Type::Executable{ name, args, .. } => commands_execution::execute_process(&name, &args, output_type),
            _ => Output::empty()
        };

        let Output{ stdout, stderr } = result;

        if self.redirection.is_none(){
            if !stdout.is_empty(){
                println!("{stdout}");
            }
            if !stderr.is_empty(){
                println!("{stderr}");
            }
            return ExecutionStatus::Continue;
        }

        let redirection = self.redirection.unwrap();
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
mod test{
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
    fn test_empty_stderr_redirection(){
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("fruits.txt");

        execute(&format!(r#"echo "apple banana orange" 2> {}"#, file_path.display()));

        assert!(file_path.exists(), "The file wasn't created");
        let content = fs::read_to_string(&file_path).unwrap();

        assert_eq!(content, "");
    }
}
