use std::env;
use std::fs;
use is_executable::is_executable;

fn any_file_in_path<F: FnMut(&str, &str) -> bool>(mut action: F){
    let Some(path_env) = env::var("PATH").ok() else { return; };

    let path_dirs = env::split_paths(&path_env);

    for dir in path_dirs{
        let entries = fs::read_dir(dir);

        if entries.is_err(){
            continue;
        }

        for entry in entries.unwrap(){
            let Ok(entry) = entry else { continue; };
            let entry = entry.path();

            if !is_executable(&entry){ continue; }
            
            let Some(filename) = entry.file_name() else { continue; };
            let Some(filename) = filename.to_str() else { continue; };
            let Some(path) = entry.to_str() else { continue; };

            if action(filename, path) { 
                return; 
            }
        }
    }
}

pub fn find_in_path(file: &str) -> Option<String>{
    let mut result = String::with_capacity(32);
    any_file_in_path(|filename, path|{
        if filename == file{
            result = path.to_string();
            return true;
        }

        false
    });

    if result.is_empty(){
        return None;
    }

    Some(result)
}

pub fn get_all_executables() -> Vec<String>{
    let mut executables = Vec::with_capacity(1024);
    any_file_in_path(|filename, _|{
        executables.push(filename.to_string());
        false
    });

    executables
}
