use std::collections::HashMap;

use rustyline::completion::{Pair, FilenameCompleter, Completer as ICompleter};
use rustyline::validate::Validator;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::{Context, Result, Helper};
use strum::VariantNames;
use lexer::Token;

use crate::shell::Shell;
use crate::trie::Trie;
use crate::{command, lexer};
use crate::env;

type CompletionResult = Result<(usize, Vec<Pair>)>;

pub struct Completer{
    commands: Trie,
    filename_completer: FilenameCompleter,
    pub completions_paths: HashMap<String, String>
}

impl Completer{
    pub fn new() -> Self{
        let executables = env::get_all_executables();
        let executables = executables.iter().map(|s| s.as_str()).collect::<Vec<_>>();

        let commands = [
            command::Kind::VARIANTS,
            &executables
        ].concat();

        Self{
            commands: Trie::from(&commands),
            filename_completer: FilenameCompleter::new(),
            completions_paths: HashMap::new()
        }
    }

    fn complete_command(&self, input: &str) -> CompletionResult{
        let mut matches = self.commands.all_starting_with(input);
        matches.sort();
        let matches = matches.iter().map(|s| {
            Pair{ 
                display: s.to_string(), 
                replacement: format!("{} ", s)
            } 
        }).collect::<Vec<_>>();

        Ok((0, matches))
    }

    fn complete_command_arguments(&self, line: &str, pos: usize) -> CompletionResult{
        let input = &line[..pos];
        let (cmd, _) = &line[..pos].split_once(' ').unwrap_or((input, ""));
        if let Some(path) = self.completions_paths.get(cmd.trim_end()){
            return self.complete_using_script(input, pos, path);
        }

        self.complete_filename(line, pos)    
    }

    fn complete_using_script(&self, input: &str, pos: usize, script_path: &str) -> CompletionResult{
        let Ok(command) = lexer::split_command(input) else {
            return Ok((pos, vec![]));
        };

        let Some(command_name) = command.first() else {
            return Ok((pos, vec![]));
        };

        let args = match &command[1..]{
            [.., prev, current] => vec![current.clone(), prev.clone()],
            [.., prev] => vec![Token::Arg("".to_string()), prev.clone()],
            _ => vec![],
        };

        let args = [vec![command_name.clone()], args].concat();

        let completions = Shell::execute_process(script_path, &args, command::OutputType::Redirect).stdout
            .split('\n')
            .map(|s| Pair{ display: s.to_string(), replacement: format!("{s} ") })
            .collect::<Vec<_>>();

        let pos = input.rfind(' ').unwrap() + 1;

        Ok((pos, completions))
    }

    fn complete_filename(&self, line: &str, pos: usize) -> CompletionResult{
        let (idx, mut matches) = self.filename_completer.complete_path(line, pos)?;

        for m in &mut matches{
            if m.replacement.ends_with('/'){
                m.display.push('/');
            }
        }

        if matches.len() == 1{
            let m = matches.pop().unwrap();

            let mut replacement = m.replacement;
            if !replacement.ends_with('/'){
                replacement.push(' ')
            }

            return Ok((idx, vec![Pair{
                display: m.display,
                replacement
            }]));
        }

        Ok((idx, matches))
    }
}

impl ICompleter for Completer{
    type Candidate = Pair;

    fn complete(
        &self, 
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> CompletionResult{
        let input = &line[..pos].trim_start();
        if input.contains(' '){
            return self.complete_command_arguments(line, pos);
        }

        self.complete_command(input)
    }
}

impl Hinter for Completer{
    type Hint = String;
}

impl Validator for Completer{}
impl Helper for Completer{}
impl Highlighter for Completer{}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_exit_match(){
        let completer = Completer::new();
        let result = completer.commands.all_starting_with("exit");
        assert!(!result.is_empty());
    }
}
