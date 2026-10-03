use std::io;
use std::path::PathBuf;

use crate::block;
use crate::config::Config;
use crate::filesystem;
use crate::html;

pub fn convert_md_to_html(source: &str) -> io::Result<PathBuf> {
    let content = filesystem::read_file(source)?;
    let config = Config::load_for(source)?;
    let elements = block::parse_with(&content, &config);
    let stem = PathBuf::from(source)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    let document = html::render_document(&elements, &stem);
    let target = PathBuf::from(filesystem::html_filename(source));
    filesystem::write_file(&target, &document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_source_returns_an_error() {
        assert!(convert_md_to_html("definitely-not-here-9d3f.md").is_err());
    }
}
