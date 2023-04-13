use arboard::Clipboard;
use std::fs::File;
use std::io::Read;
use std::process;

#[derive(Debug)]
pub struct Query {
    pub source: String,
}

impl Query {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Query, &'static str> {
        args.next();

        let source = match args.next() {
            Some(arg) => arg,
            None => return Err("No source to copy from"),
        };

        Ok(Query { source })
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
    cpy(&contents);

    Ok(())
}

pub fn cpy<'a>(contents: &'a str) {
    let mut clipboard = Clipboard::new().unwrap();

    clipboard.set_text(contents).unwrap_or_else(|err| {
        eprintln!("Couldn't copy to clipboard: {}", err);
        process::exit(1);
    });
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
