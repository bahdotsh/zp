use arboard::Clipboard;
use clap::Parser;
use is_terminal::IsTerminal;
use std::fs::File;
use std::io::{self, Read};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    author = "Gokul <@bahdotsh>",
    version = VERSION,
    about = "Tool to copy contents from a file",

)]
pub struct Zp {
    source: Option<String>,
    #[clap(short, long)]
    start: Option<usize>,
    #[clap(short, long)]
    end: Option<usize>,
}

pub struct Query {
    source: String,
    start: usize,
    end: usize,
}

impl Query {
    pub fn build(zp: Zp) -> Result<Query, &'static str> {
        let mut source = String::new();

        if io::stdout().is_terminal() && io::stderr().is_terminal() && !io::stdin().is_terminal() {
            let mut buffer = io::stdin();

            while let Ok(n) = buffer.read_to_string(&mut source) {
                if n == 0 {
                    break;
                }
            }
        } else {
            source = match zp.source {
                Some(arg) => arg,
                None => return Err("No source to copy from"),
            };
        }
        let start = match zp.start {
            Some(args) => args,
            None => "0".to_string().parse::<usize>().unwrap_or_else(|err| {
                eprintln!("Parsing error! {} ", err);
                process::exit(1);
            }),
        };
        let end = match zp.end {
            Some(args) => args,
            None => "0".to_string().parse::<usize>().unwrap_or_else(|err| {
                eprintln!("Parsing error! {} ", err);
                process::exit(1);
            }),
        };

        Ok(Query { source, start, end })
    }
}

fn read_file_content(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = match File::open(file_path) {
        Ok(content) => content,
        Err(err) => {
            return Err(err);
        }
    };
    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => return Ok(data),
        Err(err) => return Err(err),
    }
}

pub fn run(query: Query) -> Result<(), std::io::Error> {
    if io::stdout().is_terminal() && io::stderr().is_terminal() && !io::stdin().is_terminal() {
        cpy(&query.source, query.start, query.end);
    } else {
        let contents = match read_file_content(&query.source) {
            Ok(data) => data,
            Err(error) => return Err(error),
        };
        cpy(&contents, query.start, query.end);
    }
    Ok(())
}

pub fn cpy<'a>(contents: &'a str, start: usize, end: usize) {
    let mut clipboard = Clipboard::new().unwrap();

    if end == 0 as usize {
        if start == 0 as usize {
            clipboard.set_text(contents).unwrap_or_else(|err| {
                eprintln!("Couldn't copy to clipboard: {}", err);
                process::exit(1);
            });
        } else {
            let words: Vec<&str> = contents.split_whitespace().take(start).collect();
            clipboard.set_text(words.join(" ")).unwrap_or_else(|err| {
                eprintln!("Couldn't copy to clipboard: {}", err);
                process::exit(1);
            });
        }
    } else {
        let lines: Vec<&str> = contents
            .lines()
            .enumerate()
            .filter(|&(i, _)| i >= start && i <= end)
            .map(|(_, line)| line)
            .collect();
        clipboard.set_text(lines.join("\n")).unwrap_or_else(|err| {
            eprintln!("Couldn't copy to clipboard: {}", err);
            process::exit(1);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let mut clipboard = Clipboard::new().unwrap();
        let _ = cpy("Hello, world!", 0 as usize, 0 as usize);
        assert_eq!(clipboard.get_text().unwrap(), "Hello, world!");
    }
}
