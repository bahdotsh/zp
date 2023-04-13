use clap::Parser;
use std::env;
use std::process;
use zp::Query;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    author = "Gokul <@bahdotsh>",
    version = VERSION,
    about = "Tool to copy contents from a file",

)]
struct Zp {
    ///
    file_name: Option<String>,
    start: Option<usize>,
    end: Option<usize>,
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
