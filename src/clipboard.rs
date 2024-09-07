use arboard::Clipboard;
use std::process;

pub fn cpy<'a>(contents: &'a str, start: usize, end: usize) {
    let mut clipboard = Clipboard::new().unwrap();

    if end == 0 {
        if start == 0 {
            clipboard.set_text(contents).unwrap_or_else(|err| {
                eprintln!("Couldn't copy to clipboard: {}", err);
                process::exit(1);
            });
        } else {
            let words: Vec<&str> = contents.split_whitespace().take(start).collect();
            clipboard.set_text(words.join(" ")).unwrap_or_else(|err| {
                eprintln!("Couldn't copy to clipboard: {}", err);
                process::exit(1);
            });
        }
    } else {
        let lines: Vec<&str> = contents
            .lines()
            .enumerate()
            .filter(|&(i, _)| i >= start && i <= end)
            .map(|(_, line)| line)
            .collect();
        clipboard.set_text(lines.join("\n")).unwrap_or_else(|err| {
            eprintln!("Couldn't copy to clipboard: {}", err);
            process::exit(1);
        });
    }
}
