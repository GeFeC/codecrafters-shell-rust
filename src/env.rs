use std::env;
use std::fs;
use is_executable::is_executable;

pub fn find_in_path(file: &str) -> Option<String>{
    let path_env = env::var("PATH").expect("Unable to read $PATH variable");
    let path_dirs = env::split_paths(&path_env);

    for dir in path_dirs{
        let paths = fs::read_dir(dir);

        if paths.is_err(){
            continue;
        }

        for path in paths.unwrap(){
            let path = path.ok()?.path();
            if path.is_file() && is_executable(&path) && path.file_name()?.to_str()? == file{
                return Some(path.to_str()?.to_owned());
            }
        }
    }

    None
}
