use crate::command::CommandArgs;
use crate::command::Output;
use std::process;

pub struct Program{
    command: process::Command
}

impl Program{
    pub fn new(name: &str) -> Self{
        Self{
            command: process::Command::new(name)
        }
    }

    pub fn args(mut self, args: &CommandArgs) -> Self{
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

    pub fn spawn_and_wait(&mut self){
        if let Err(err) = self.command.spawn().and_then(|mut c| c.wait()){
            println!("{err}");
        }
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
