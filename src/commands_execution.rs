use crate::command::{Command, CommandArgs};
use std::env;
use std::path::Path;

pub fn execute_echo(args: &CommandArgs) -> String{
    args.join(" ").to_string()
}

pub fn execute_pwd() -> String{
    format!("{}", env::current_dir().unwrap().display())
}

pub fn execute_cd(args: &CommandArgs) -> String{
    let home_dir = env::home_dir();

    let path = args.join(" ");
    let path = match path.as_str(){
        "~" | "" => home_dir.as_deref().and_then(Path::to_str).unwrap_or("~").to_owned(),
        _ => path
    };

    if env::set_current_dir(&path).is_err(){
        return format!("cd: {}: No such file or directory", path);
    }

    String::new()
}

pub fn execute_type(checked_cmd: &CommandArgs) -> String{
    if checked_cmd.len() != 1{
        return String::from("Usage: type <command name without args>");
    }

    let cmd_name = checked_cmd.first().unwrap();
    let command = Command::from_str(cmd_name);

    if command.is_err(){
        let executable_path = crate::env::find_in_path(cmd_name);

        match executable_path{
            Some(path) => { return format!("{} is {}", cmd_name, path); },
            None => {
                return format!("{}: not found", cmd_name);
            }
        }
    }

    format!("{} is a shell builtin", cmd_name)
}


