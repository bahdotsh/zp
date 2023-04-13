use arboard::Clipboard;
use std::error::Error;
use std::fs;
use std::process;

#[derive(Debug)]
pub struct Query {
    pub source: String,
}

impl Query {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Query, &'static str> {
        args.next();
        
        let source = match args.next() {
            Some(arg) => arg,
            None => return Err("No source to copy from"),
        };

        Ok(Query { source })
    }
}

pub fn run(query: Query) -> Result<(), Box<dyn Error>> {
    let contents =
        fs::read_to_string(query.source).expect("Should have been able to read the file");

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
        let _ = cpy("Hello, world!");
        assert_eq!(Clipboard::get_text().unwrap(), "Hello, world!");
    }
}
