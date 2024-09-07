use clap::Parser;
use std::process;
use zp::{Query, Zp};

fn main() {
    // Parse command-line arguments into a Zp struct and build the query
    let query = Query::build(Zp::parse()).unwrap_or_else(|err| {
        eprintln!("Problem parsing query: {}", err);
        process::exit(1);
    });

    // Run the application with the query
    if let Err(e) = zp::run(query) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
