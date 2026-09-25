mod cli;
mod conversion;
mod filesystem;
mod types;
fn main() {
    // Get File Name
    if let Some(file) = cli::collect_filename() {
        // Check File Existence
        if let Some(path) = filesystem::check_file_exists(file) {
            // Convert File To MD
            conversion::convert_md_to_html(path);
        }
    }
}
