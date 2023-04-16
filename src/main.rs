use clap::Parser;
use std::env;
use std::process;
use zp::{Query, Zp};

fn main() {

    //let query = Query::build(env::args()).unwrap_or_else(|err| {
      //  eprintln!("Problem parsing query: {err}");
        //process::exit(1);
    //});

    let query = Query::build(Zp::parse()).unwrap_or_else(|err| {
        eprintln!("Problem parsing query: {}", err);
        process::exit(1);
    });

    if let Err(e) = zp::run(query) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
