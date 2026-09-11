use crate::command::Output;

use crate::command::Command;

use crate::{lexer::{self, Token}, shell::Shell};

impl Shell{
    pub fn execute_command_pipeline(&mut self, command_pipeline: &[lexer::Token]) -> Output{
        let commands = command_pipeline.split(|t| matches!(t, Token::Pipe));

        let mut input;
        let mut output = String::with_capacity(1024);
        for cmd in commands{
            input = lexer::split_command(&output).expect("TODO");
            let cmd = Command::from_tokens(&[cmd, &input].concat()).expect("TODO");
            let Output{ stdout, .. } = self.command_output(&cmd);
            output = stdout;
        }

        println!("{output}");

        Output::empty()
    }
}
