use crate::command::Output;
use crate::lexer::CommandLine;
use std::error::Error;
use std::process;
use std::process::Child;

pub struct Program{
    pub command: process::Command
}

impl Program{
    pub fn new(name: &str) -> Self{
        Self{
            command: process::Command::new(name)
        }
    }

    pub fn args(mut self, args: &CommandLine) -> Self{
        if !args.is_empty(){
            self.command.args(args.iter().map(|t| t.to_string()));
        }
        self
    }

    pub fn envs(mut self, envs: &[(String, String)]) -> Self{
        for (varname, value) in envs{
          self.command.env(varname, value);
        }

        self
    }

    pub fn spawn(&mut self) -> Result<Child, impl Error>{
        self.command.spawn()
    }

    fn stringify_output(output: &[u8]) -> String{
        std::str::from_utf8(output).unwrap().trim().to_string()
    }

    pub fn output(&mut self) -> Output{
        match self.command.output(){
            Ok(output) => Output{
                stdout: Self::stringify_output(&output.stdout),
                stderr: Self::stringify_output(&output.stderr),
            },
            Err(err) => Output{
                stdout: String::new(),
                stderr: format!("{err}")
            }
        }
    }
}
