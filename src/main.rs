use cpy::Query;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let query = Query::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing query: {err}");
        process::exit(1);
    });

    if let Err(e) = cpy::run(query) {
        println!("Application error: {e}");
        process::exit(1);
    }
}
