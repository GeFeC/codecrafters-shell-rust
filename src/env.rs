use std::env;
use std::fs;
use is_executable::is_executable;

pub fn find_in_path(file: &str) -> Option<String>{
    let path_env = env::var("PATH").ok()?;
    let path_dirs = env::split_paths(&path_env);

    for dir in path_dirs{
        let entries = fs::read_dir(dir);

        if entries.is_err(){
            continue;
        }

        for entry in entries.unwrap(){
            let entry = entry.ok()?.path();
            if entry.is_file() && is_executable(&entry) && entry.file_name()?.to_str()? == file{
                return Some(entry.to_str()?.to_owned());
            }
        }
    }

    None
}
