use rustyline::completion::{Completer, Pair};
use rustyline::validate::Validator;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::{Context, Result, Helper};
use strum::VariantNames;

use crate::command::Kind;

pub struct CommandCompleter{
    commands: Vec<String>
}

impl CommandCompleter{
    pub fn new() -> Self{
        Self{
            commands: Kind::VARIANTS.iter().map(|&s| s.to_string()).collect()
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

        let mut matches = Vec::new();
        for cmd in &self.commands{
            if cmd.starts_with(input){
                matches.push(Pair{
                    display: cmd.clone(),
                    replacement: cmd.clone()
                })
            }
        }

        Ok((0, matches))
    }
}

impl Hinter for CommandCompleter{
    type Hint = String;
}

impl Validator for CommandCompleter{}
impl Helper for CommandCompleter{}
impl Highlighter for CommandCompleter{}


