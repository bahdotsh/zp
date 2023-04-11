use cli_clipboard;
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct Query {
    pub source: String,
}

impl Query {
    pub fn build(args: &[String]) -> Result<Query, &'static str> {
        if args.len() == 1 || args.len() > 2 {
            return Err("Invalid Query");
        }
        let source = args[1].clone();

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
        let _ = cpy("Hello, world!");
        assert_eq!(cli_clipboard::get_contents().unwrap(), "Hello, world!");
    }
}
