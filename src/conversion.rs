use crate::filesystem;

pub fn convert_md_to_html(filepath: String) {
    if let Some(file_content) = filesystem::get_file_content(filepath) {
        // println!("{}", file_content);
        let lines = convert_string_to_lines(&file_content);
        // println!("{:?}", lines);
        for line in lines {
            let words = convert_string_to_words(line);
            println!("{:?}", words);
        }
    }
}

fn convert_string_to_lines(content: &str) -> Vec<&str> {
    let lines = content.lines().collect();
    lines
}

fn convert_string_to_words(content: &str) -> Vec<&str> {
    let words = content.split_whitespace().collect();
    words
}

// fn convert_to_bold(text: String) -> String {
//     format!("<b>{}</b>", text)
// }

// fn convert_to_italic(text: String) -> String {
//     format!("<i>{}</i>", text)
// }

// fn convert_to_highlight(text: String) -> String {
//     format!("<mark>{}</mark>", text)
// }

// fn convert_to_underline(text: String) -> String {
//     format!("<u>{}</u>", text)
// }

// fn convert_to_inlinecode(text: String) -> String {
//     format!("<code>{}</code>", text)
// }

// fn convert_to_strikethrough(text: String) -> String {
//     format!("<s>{}</s>", text)
// }
