use crate::filesystem;

pub fn convert_md_to_html(filepath: String) {
    if let Some(file_content) = filesystem::get_file_content(filepath) {
        println!("{}", file_content);
    }
}
