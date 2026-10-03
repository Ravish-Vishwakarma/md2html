use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

pub fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn html_filename(source: &str) -> String {
    let stem = Path::new(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    format!("{stem}.html")
}

pub fn write_file(path: &Path, content: &str) -> io::Result<PathBuf> {
    fs::write(path, content)?;
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_the_extension_for_html() {
        assert_eq!(html_filename("notes.md"), "notes.html");
        assert_eq!(html_filename("docs/readme.md"), "readme.html");
        assert_eq!(html_filename("archive.tar.md"), "archive.tar.html");
    }

    #[test]
    fn missing_files_are_reported_as_absent() {
        assert!(!file_exists("definitely-not-here-9d3f.md"));
    }
}
