use std::collections::HashMap;

use rustyline::completion::{Pair, FilenameCompleter, Completer as ICompleter};
use rustyline::validate::Validator;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::{Context, Result, Helper};
use strum::VariantNames;

use crate::shell::Shell;
use crate::trie::Trie;
use crate::command;
use crate::env;

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
}

impl ICompleter for Completer{
    type Candidate = Pair;

    fn complete(
        &self, 
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)>{

        let input = &line[..pos].trim_start();
        if input.contains(' '){
            if let Some(path) = self.completions_paths.get(input.trim_end()){
                let completions = Shell::execute_process(path, &[], command::OutputType::Redirect).stdout
                    .split('\n')
                    .map(|s| Pair{ display: s.to_string(), replacement: format!("{s} ") })
                    .collect::<Vec<_>>();

                return Ok((pos, completions));
            }

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

            return Ok((idx, matches));
        } 

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
