use crate::clipboard::cpy;
use crate::file::read_file_content;
use crate::query::Query;
use is_terminal::IsTerminal;
use std::io;

pub fn run(query: Query) -> Result<(), std::io::Error> {
    if io::stdout().is_terminal() && io::stderr().is_terminal() && !io::stdin().is_terminal() {
        cpy(&query.source, query.start, query.end);
    } else {
        let contents = read_file_content(&query.source)?;
        cpy(&contents, query.start, query.end);
    }
    Ok(())
}
