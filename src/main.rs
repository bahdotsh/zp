use cpy::Query;
use std::env;
use std::process;

fn main() {
    let query = Query::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing query: {err}");
        process::exit(1);
    });

    if let Err(e) = cpy::run(query) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
