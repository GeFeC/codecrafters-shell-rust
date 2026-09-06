use rustyline::completion::{Completer, Pair};
use rustyline::validate::Validator;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::{Context, Result, Helper};
use strum::VariantNames;

use crate::trie::Trie;
use crate::command;
use crate::env;

pub struct CommandCompleter{
    commands: Trie
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
            commands: Trie::from(&commands)
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
        let input = &line[..pos];

        let matches = self.commands.all_starting_with(input);
        let matches = matches.iter().map(|s| {
            let s = format!("{s} "); 
            Pair{ 
                display: s.clone(), 
                replacement: s
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
