mod cli;
mod conversion;
mod filesystem;
fn main() {
    if let Some(file) = cli::collect_filename() {
        if let Some(path) = filesystem::check_file_exists(file) {
            conversion::convert_md_to_html(path);
        }
    }
}
