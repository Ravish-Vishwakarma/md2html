use std::env;
use std::path::Path;

pub const USAGE: &str = "md2html 0.1.0
Convert Markdown files to standalone HTML pages.

USAGE:
    md2html <file.md> [more.md ...]

OPTIONS:
    -h, --help    Show this message

Each input is converted to <name>.html in the current directory.";

pub enum Cli {
    Run(Vec<String>),
    ShowHelp,
    Invalid,
}

pub fn parse() -> Cli {
    let arguments: Vec<String> = env::args().skip(1).collect();

    if arguments.is_empty() {
        eprintln!("error: no input files given");
        return Cli::Invalid;
    }

    let mut files: Vec<String> = Vec::new();

    for argument in arguments {
        match argument.as_str() {
            "-h" | "--help" => return Cli::ShowHelp,
            flag if flag.starts_with('-') => {
                eprintln!("error: unknown option `{flag}`");
                return Cli::Invalid;
            }
            _ => {}
        }

        if is_markdown(&argument) {
            files.push(argument);
        } else {
            eprintln!("error: `{argument}` is not a markdown file (expected .md)");
        }
    }

    if files.is_empty() {
        Cli::Invalid
    } else {
        Cli::Run(files)
    }
}

fn is_markdown(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_markdown_extensions() {
        assert!(is_markdown("notes.md"));
        assert!(is_markdown("NOTES.MD"));
        assert!(is_markdown("docs/readme.markdown.md"));
        assert!(!is_markdown("notes.txt"));
        assert!(!is_markdown("notes"));
        assert!(!is_markdown("md"));
    }
}
