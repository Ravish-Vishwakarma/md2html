use crate::filesystem;

pub fn convert_md_to_html(filepath: String) {
    if let Some(file_content) = filesystem::get_file_content(filepath) {
        println!("{}", file_content);
    }
}

fn convert_to_bold(text: String) -> String {
    format!("<b>{}</b>", text)
}

fn convert_to_italic(text: String) -> String {
    format!("<i>{}</i>", text)
}

fn convert_to_highlight(text: String) -> String {
    format!("<mark>{}</mark>", text)
}

fn convert_to_underline(text: String) -> String {
    format!("<u>{}</u>", text)
}

fn convert_to_inlinecode(text: String) -> String {
    format!("<code>{}</code>", text)
}

fn convert_to_strikethrough(text: String) -> String {
    format!("<s>{}</s>", text)
}
