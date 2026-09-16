use std::env;
pub fn collect_filename() -> Option<String> {
    let filename: Option<String> = env::args().nth(1);
    if filename.is_none() {
        println!("Please Provide a Filename");
    }
    match filename {
        Some(file) => {
            if file.to_lowercase().ends_with(".md") {
                return Some(file);
            } else {
                println!("Please enter a markdown file which ends with .md");
                return None;
            }
        }
        None => None,
    }
}
