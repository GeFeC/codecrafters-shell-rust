use crate::shell::Shell;

impl Shell{
    pub fn insert_variables_values(&mut self, arg: String) -> String{
        let mut result = String::with_capacity(arg.len());

        let mut arg = &arg[..];

        while !arg.is_empty(){
            let dollar_pos = arg.find('$').unwrap_or(arg.len());
            result.push_str(&arg[0..dollar_pos]);
            if dollar_pos == arg.len() {
                return result;
            }

            arg = &arg[(dollar_pos + 1)..];
            let var_end = arg
                .chars()
                .into_iter()
                .position(|c| c == ' ' || c == '$')
                .unwrap_or(arg.len());

            if var_end == 0{
                result.push('$');
            }
            else {
                let var_name = &arg[..var_end];
                let var_value = self.variables.get(var_name);
                result.push_str(var_value.unwrap_or(&String::new()));
            }

            arg = &arg[var_end..];
        }
        
        result
    }
}

#[cfg(test)]
mod tests{
    use std::collections::HashMap;

use super::*;

    #[test]
    fn test_variables(){
        let mut shell = Shell::new().unwrap();
        shell.variables = HashMap::from([
            ("hello".to_owned(), "world".to_owned()),
            ("pi".to_owned(), "3.1415".to_owned()),
        ]);

        let mut test = |got: &str, expected: &str|{
            assert_eq!(shell.insert_variables_values(got.to_owned()), expected);
        };

        test("$", "$");
        test("$hello", "world");
        test("$hello$hello", "worldworld");
        test("$hello $hello", "world world");
        test("$hello$pi", "world3.1415");
    }
}
