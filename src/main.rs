use zp::Query;
use std::env;
use std::process;
use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Copy file contents 
#[derive(Parser)]
#[command(
    author = "Gokul <@bahdotsh>",
    version = VERSION,
    about = "Tool to copy contents from a file",

)]
struct Zp {
    ///Name of the file to copy from.
    file_name: String,
}

fn main() {
    Zp::parse();
    let query = Query::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing query: {err}");
        process::exit(1);
    });

    if let Err(e) = zp::run(query) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
