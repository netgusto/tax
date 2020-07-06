#[macro_use]
extern crate lazy_static;
extern crate dirs;

use regex::Regex;
use std::fs;
use std::path::Path;

/*
TODO:
* zsh binding => fzf like interface to view and manage tasks (input, delete, mark as completed)
* improve display of prompt line
* tmux status bar compat?
*/

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

fn run_app() -> Result<(), ()> {
    let home = match dirs::home_dir() {
        None => return Err(()),
        Some(h) => h,
    };

    let file_path = home.join(Path::new("taxfile"));
    let str_file_path = file_path.to_str().unwrap();

    let contents = fs::read_to_string(str_file_path)
        .expect(format!("Something went wrong reading the file {:?}", str_file_path).as_str());
    if contents.trim().len() == 0 {
        println!("Nothing to do");
        return Ok(());
    }

    match next_task(contents) {
        None => return Ok(()),
        Some(task) => println!("{}", task),
    }

    Ok(())
}

fn next_task(contents: String) -> Option<String> {
    // Find lines matching pattern
    // - [ ] Task name
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^\s*(?:-|\*)\s+\[\s\]\s+(.+?)$").unwrap();
    }

    for cap in RE.captures_iter(contents.as_str()) {
        return Some(format!("⏭️ {}", String::from(&cap[1])));
    }

    None
}
