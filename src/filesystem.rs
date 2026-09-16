use std::env;
use std::fs;
use std::path::PathBuf;
fn get_current_path() -> Option<String> {
    match env::current_dir() {
        Ok(path) => Some(path.to_string_lossy().into_owned()),
        Err(_) => None,
    }
}

fn combine_folder_and_file(folder: String, file: String) -> String {
    let mut full_path = PathBuf::from(folder);
    full_path.push(file);
    let filepath: String = full_path.display().to_string();
    filepath
}

pub fn check_file_exists(filename: String) -> Option<String> {
    let dir_path = get_current_path();
    match dir_path {
        Some(dpath) => {
            let fullpath = combine_folder_and_file(dpath, filename);
            match fs::exists(&fullpath) {
                Ok(true) => Some(fullpath),
                Ok(false) => {
                    println!("File Doesn't Exists at: {}", fullpath);
                    None
                }
                Err(_) => {
                    println!("Error: Cannot Check If File Exists");
                    None
                }
            }
        }
        None => None,
    }
}

pub fn get_file_content(filepath: String) -> Option<String> {
    match fs::read_to_string(filepath) {
        Ok(content) => Some(content),
        Err(_) => {
            println!("Error: Cannot reading file content");
            None
        }
    }
}

// pub fn save_content_to_html(content: String) {}
