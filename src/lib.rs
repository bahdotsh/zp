use cli_clipboard;
use std::error::Error;
use std::fs;

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

    cpy(&contents)
}

pub fn cpy<'a>(contents: &'a str) -> Result<(), Box<dyn Error>> {
    cli_clipboard::set_contents(contents.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let mut ctx = ClipboardContext::new().unwrap();
        assert_eq!(ctx.get_contents().unwrap(), "Hello, world!");
    }
}
