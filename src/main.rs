use std::env;
use cli_clipboard;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let query = Query::new(&args);


    let contents = fs::read_to_string(query.source)
        .expect("Should have been able to read the file");

    cli_clipboard::set_contents(contents.to_owned()).unwrap();

}

#[derive(Debug)]
struct Query {
    source: String,
}

impl Query {
    fn new(args: &[String]) -> Query {
        if args.len() == 1 || args.len() > 2{
            panic!("Invalid Query");
        }
            let source = args[1].clone();

            Query { source }
    }
}

