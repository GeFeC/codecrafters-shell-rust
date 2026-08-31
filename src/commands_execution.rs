use crate::command::{Command, CommandArgs};
use std::env;
use std::path::Path;

pub fn execute_echo(args: CommandArgs){
    println!("{}", args.join(" "));
}

pub fn execute_pwd(){
    println!("{}", env::current_dir().unwrap().display());
}

pub fn execute_cd(args: CommandArgs){
    let home_dir = env::home_dir();

    let path = args.join(" ");
    let path = match path.as_str(){
        "~" | "" => home_dir.as_deref().and_then(Path::to_str).unwrap_or("~").to_owned(),
        _ => path
    };

    env::set_current_dir(&path)
        .unwrap_or_else(|_| println!("cd: {}: No such file or directory", path));
}

pub fn execute_type(checked_cmd: CommandArgs){
    if checked_cmd.len() != 1{
        println!("Usage: type <command name without args>");
        return;
    }

    let cmd_name = checked_cmd.first().unwrap();
    let command = Command::from(cmd_name, &[]);

    if command.is_none(){
        let executable_path = crate::env::find_in_path(cmd_name);

        match executable_path{
            Some(path) => println!("{} is {}", cmd_name, path),
            None => {
                println!("{}: not found", cmd_name);
            }
        }

        return;
    }

    println!("{} is a shell builtin", cmd_name);
}


