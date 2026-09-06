use rustyline::completion::{Completer, Pair, FilenameCompleter};
use rustyline::validate::Validator;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::{Context, Result, Helper};
use strum::VariantNames;

use crate::trie::Trie;
use crate::command;
use crate::env;

pub struct CommandCompleter{
    commands: Trie,
    filename_completer: FilenameCompleter
}

impl CommandCompleter{
    pub fn new() -> Self{
        let executables = env::get_all_executables();
        let executables = executables.iter().map(|s| s.as_str()).collect::<Vec<_>>();

        let commands = [
            command::Kind::VARIANTS,
            &executables
        ].concat();

        Self{
            commands: Trie::from(&commands),
            filename_completer: FilenameCompleter::new()
        }
    }
}

impl Completer for CommandCompleter{
    type Candidate = Pair;

    fn complete(
        &self, 
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)>{

        let input = &line[..pos].trim_start();
        if input.contains(' '){
            return self.filename_completer.complete_path(line, pos);
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

impl Hinter for CommandCompleter{
    type Hint = String;
}

impl Validator for CommandCompleter{}
impl Helper for CommandCompleter{}
impl Highlighter for CommandCompleter{}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_exit_match(){
        let completer = CommandCompleter::new();
        let result = completer.commands.all_starting_with("exit");
        assert!(!result.is_empty());
    }
}
