use std::env;
use cli_clipboard;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let source = &args[1];

    println!("Copying from: {}", source);

    let contents = fs::read_to_string(source)
        .expect("Should have been able to read the file");

    cli_clipboard::set_contents(contents.to_owned()).unwrap();

}

