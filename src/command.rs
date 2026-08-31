use crate::commands_execution;

pub type CommandArgs<'a> = &'a[&'a str];
pub type CommandArgsStorage<'a> = Vec<&'a str>;

pub enum Command<'a>{
    Exit,
    Pwd,
    Echo(CommandArgs<'a>),
    Type(CommandArgs<'a>),
    Cd(CommandArgs<'a>)
}

impl Command<'_>{
    pub fn from<'a>(command: &'a str, rest: CommandArgs<'a>) -> Option<Command<'a>>{
        match command{
            "exit" => Some(Command::Exit),
            "echo" => Some(Command::Echo(rest)),
            "type" => Some(Command::Type(rest)),
            "pwd" => Some(Command::Pwd),
            "cd" => Some(Command::Cd(rest)),
            _ => None
        }
    }

    pub fn execute(self) -> ExecutionStatus{
        match self{
            Command::Exit => return ExecutionStatus::Exit,
            Command::Echo(args) => commands_execution::execute_echo(args),
            Command::Type(args) => commands_execution::execute_type(args),
            Command::Pwd => commands_execution::execute_pwd(),
            Command::Cd(args) => commands_execution::execute_cd(args),
        };

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

