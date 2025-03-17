use crate::clipboard::cpy;
use crate::file::read_file_content;
use crate::query::{Query, Zp};
use is_terminal::IsTerminal;
use std::io;

pub fn run(zp: Zp) -> Result<(), std::io::Error> {
    let query = Query::build(&zp).unwrap();
    if io::stdout().is_terminal() && io::stderr().is_terminal() && !io::stdin().is_terminal() {
        cpy(&query.source, query.start, query.end);
    } else {
        let contents = read_file_content(&query.source)?;
        cpy(&contents, query.start, query.end);
    }

    Ok(())
}
