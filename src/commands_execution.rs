use crate::command::{Command, CommandArgs, OutputType, Output};
use crate::command;
use crate::lexer::{ tokens_to_string };
use std::env;
use std::path::Path;

pub fn execute_echo(args: &CommandArgs) -> Output{
    Output::stdout(&tokens_to_string(args))
}

pub fn execute_pwd() -> Output{
    Output::stdout(&format!("{}", env::current_dir().unwrap().display()))
}

pub fn execute_cd(args: &CommandArgs) -> Output{
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

pub fn execute_type(checked_cmd: &CommandArgs) -> Output{
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

pub fn execute_process(executable: &str, args: &CommandArgs, output_type: OutputType) -> Output{
    let mut command = std::process::Command::new(executable);
    if !args.is_empty(){
        command.args(args.iter().map(|t| t.to_string()));
    }
    
    match output_type{
        OutputType::Redirect => {
            match command.output(){
                Ok(output) => Output{
                        stdout: std::str::from_utf8(&output.stdout).unwrap().trim().to_string(),
                        stderr: std::str::from_utf8(&output.stderr).unwrap().trim().to_string()
                    },

                Err(err) => Output{
                    stdout: String::new(),
                    stderr: format!("{err}")
                }
            }
        },
        OutputType::Display => {
            match command.spawn().and_then(|mut c| c.wait()){
                Ok(_) => { Output::empty() },
                Err(err) => { println!("{err}"); Output::empty() }
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::lexer::Token;

    fn test_type_helper(cmd: &str) -> Output{
        execute_type(&[Token::Arg(cmd.to_owned())])
    }

    #[test]
    fn test_type(){
        let output = test_type_helper("cat");
        assert_ne!(output.stdout, "cat is a shell builtin");

        let output = test_type_helper("echo");
        assert_eq!(output.stdout, "echo is a shell builtin");
    }
}
