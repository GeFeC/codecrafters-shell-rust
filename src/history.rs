use std::io::Write;
use std::fs::OpenOptions;

use crate::shell::Shell;

impl Shell{
    pub fn save_history(&mut self, filepath: &str, should_append: bool) -> Result<(), String>{
        let saving_status = OpenOptions::new()
            .write(true)
            .create(true)
            .append(should_append)
            .truncate(!should_append)
            .open(filepath)
            .and_then(|mut f|{
                for entry in self.editor.history(){
                    writeln!(f, "{entry}")?;
                }

                Ok(())
            });

        saving_status.map_err(|err| format!("{err}"))
    }

    pub fn append_history(&mut self, filepath: &str) -> Result<(), String>{
        self.save_history(filepath, true)
    }

    pub fn write_history(&mut self, filepath: &str) -> Result<(), String>{
        self.save_history(filepath, false)
    }
}
