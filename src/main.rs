use clap::Parser;
use std::process;
use zp::history::print_clipboard_history;
use zp::{Query, Zp};

fn main() {
    // Parse command-line arguments into a Zp struct
    let zp = Zp::parse();
    if zp.log {
        print_clipboard_history().unwrap();
    } else {
        match Query::build(&zp) {
            Ok(_) => {
                // Run the application with the Zp struct (passing the original parsed Zp)
                if let Err(e) = zp::run(zp) {
                    eprintln!("Application error: {e}");
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("Problem parsing query: {}", err);
                process::exit(1);
            }
        }
    }
}
