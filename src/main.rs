mod block;
mod cli;
mod config;
mod conversion;
mod filesystem;
mod helper;
mod html;
mod inline;
mod types;

use cli::Cli;

fn main() {
    let files = match cli::parse() {
        Cli::Run(files) => files,
        Cli::ShowHelp => {
            println!("{}", cli::USAGE);
            return;
        }
        Cli::Invalid => {
            eprintln!("\n{}", cli::USAGE);
            std::process::exit(2);
        }
    };

    let mut failures = 0;

    for file in files {
        if !filesystem::file_exists(&file) {
            eprintln!("error: no such file: {file}");
            failures += 1;
            continue;
        }

        match conversion::convert_md_to_html(&file) {
            Ok(path) => println!("{} -> {}", file, path.display()),
            Err(error) => {
                eprintln!("error: could not convert {file}: {error}");
                failures += 1;
            }
        }
    }

    if failures > 0 {
        std::process::exit(1);
    }
}
