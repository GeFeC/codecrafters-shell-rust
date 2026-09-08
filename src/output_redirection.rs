use crate::command::{ Output };
use crate::lexer;
use std::fs::{ OpenOptions };
use std::io::Write;

pub struct Redirection{
    pub target: RedirectionTarget,
    pub r#type: RedirectionType
}

pub enum RedirectionTarget{
    Stdout(String),
    Stderr(String)
}

pub enum RedirectionType{
    Overwrite,
    Append
}

pub fn get_redirection_and_new_command_args(command_args: Vec<lexer::Token>) -> (Option<Redirection>, Vec<lexer::Token>){
    let symbol_pos = command_args.iter().position(|e| e.is_redirection());

    let (symbol, command_args, target_file) = match symbol_pos{
        Some(idx) => {
            let symbol = &command_args[idx];
            let (command_args, target_file) = command_args.split_at(idx);
            let target_file = &target_file[1..];

            (Some(symbol), command_args.to_vec(), target_file.to_vec())
        },
        None => (None, command_args, Vec::<Token>::new())
    };

    if symbol.is_none(){
        return (None, command_args.to_vec());
    }

    let symbol = symbol.unwrap();
    use lexer::Token;

    let Some(Token::Arg(target_file)) = target_file.into_iter().next() else{ 
        unreachable!("Wrong redirection file name not handled!");
    };

    let target = match symbol{
        Token::Stdout | Token::StdoutAppend => RedirectionTarget::Stdout(target_file),
        Token::Stderr | Token::StderrAppend => RedirectionTarget::Stderr(target_file),
        _ => unreachable!()
    };
    
    let r#type = match symbol{
        Token::Stdout | Token::Stderr => RedirectionType::Overwrite,
        Token::StdoutAppend | Token::StderrAppend => RedirectionType::Append,
        _ => unreachable!()
    };

    (Some(Redirection{ target, r#type }), command_args)
}

impl Output{
    pub fn redirect(&self, redirection: Redirection){
        if !matches!(redirection.target, RedirectionTarget::Stdout(_)) && !self.stdout.is_empty(){
            println!("{}", self.stdout);
        }

        if !matches!(redirection.target, RedirectionTarget::Stderr(_)) && !self.stderr.is_empty(){
            println!("{}", self.stderr);
        }

        let (out, file) = match redirection.target{
            RedirectionTarget::Stdout(file) => (&self.stdout, file),
            RedirectionTarget::Stderr(file) => (&self.stderr, file),
        };

        let should_append = matches!(redirection.r#type, RedirectionType::Append);
        let file = OpenOptions::new()
            .write(true)
            .append(should_append)
            .truncate(!should_append)
            .create(true)
            .open(file)
            .and_then(|mut f|{
                write!(f, "{out}")?;
                if !out.is_empty() && !out.ends_with("\n"){
                    return writeln!(f);
                }

                Ok(())
            });

        if let Err(err) = file{
            println!("{}", err);
        }
    }
}
