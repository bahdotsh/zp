use arboard::Clipboard;
use atty::Stream;
use std::fs::File;
use std::process;
use std::io::{self, Read};
use atty::Stream;

#[derive(Debug)]
pub struct Query {
    pub source: String,
    pub start: usize,
    pub end: usize,
}

impl Query {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Query, &'static str> {
        args.next();
        if atty::is(Stream::Stdout) && atty::is(Stream::Stderr) && atty::isnt(Stream::Stdin) {
            let mut buffer = io::stdin();
            let mut contents = String::new();
            let start = match args.next() {
                Some(arg) => arg,
                None => "0".to_string(),
            }
            .parse::<usize>()
            .unwrap_or_else(|err| {
                eprintln!("Parsing error! {} ", err);
                process::exit(1);
            });

            let end = match args.next() {
                Some(arg) => arg,
                None => "0".to_string(),
            }
            .parse::<usize>()
            .unwrap_or_else(|err| {
                eprintln!("Parsing error! {}", err);
                process::exit(1);
            });

            while let Ok(n) = buffer.read_to_string(&mut contents) {
                if n == 0 {
                    break;
                }
            }
            cpy(&contents, start as usize, end as usize);
            process::exit(1);
        }

        let source = match args.next() {
            Some(args) => args,
            None => return Err("No source to copy from"),
        };

        let start = match args.next() {
            Some(arg) => arg,
            None => "0".to_string(),
        }
        .parse::<usize>()
        .unwrap_or_else(|err| {
            eprintln!("Parsing error! {} ", err);
            process::exit(1);
        });

        let end = match args.next() {
            Some(arg) => arg,
            None => "0".to_string(),
        }
        .parse::<usize>()
        .unwrap_or_else(|err| {
            eprintln!("Parsing error! {}", err);
            process::exit(1);
        });

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
    let contents = match read_file_content(&query.source) {
        Ok(data) => data,
        Err(error) => return Err(error),
    };
    cpy(&contents, query.start, query.end);

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
        let _ = cpy("Hello, world!");
        assert_eq!(clipboard.get_text().unwrap(), "Hello, world!");
    }
}
