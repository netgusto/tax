#[macro_use]
extern crate lazy_static;
extern crate dirs;

use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

fn main() {
    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}

fn run_app() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].as_str();
        if cmd == "edit" {
            return cmd_edit();
        } else {
            return Err(format!(
                "Unknown command \"{}\"; the sole valid command is \"edit\"",
                args[1]
            ));
        }
    }

    cmd_display()
}

fn cmd_display() -> Result<(), String> {
    let str_file_path = get_file_path().unwrap();

    let contents = fs::read_to_string(&str_file_path)
        .expect(format!("Something went wrong reading the file {:?}", str_file_path).as_str());

    if contents.trim().len() == 0 {
        return Ok(());
    }

    match next_task(contents) {
        Some(task) => println!("{}", task),
        None => {
            return Ok(());
        }
    }

    Ok(())
}

fn cmd_edit() -> Result<(), String> {
    let str_file_path = get_file_path().unwrap();
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(format!("/usr/bin/env \"$EDITOR\" \"{}\"", str_file_path));

    if let Ok(mut child) = cmd.spawn() {
        match child.wait() {
            Ok(_) => (),
            Err(_) => {
                return Err(String::from("Could not run $EDITOR"));
            }
        }
    } else {
        return Err(String::from("Could not run $EDITOR"));
    }

    return Ok(());
}

fn home_dir() -> Result<PathBuf, String> {
    match dirs::home_dir() {
        None => return Err(String::from("Cannot find home dir")),
        Some(h) => Ok(h),
    }
}

fn get_file_path() -> Result<String, String> {
    let home = match home_dir() {
        Err(e) => {
            return Err(e);
        }
        Ok(home) => home,
    };
    let file_path = home.join(Path::new("taxfile"));
    return Ok(String::from(file_path.to_str().unwrap()));
}

fn next_task(contents: String) -> Option<String> {
    // Find lines matching pattern
    // - [ ] Task name
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^\s*(?:-|\*)\s+\[\s\]\s+(.+?)$").unwrap();
    }

    let mut tasks: Vec<String> = Vec::new();

    for cap in RE.captures_iter(contents.as_str()) {
        tasks.push(format!("{}", String::from(&cap[1])));
    }

    if tasks.len() == 0 {
        return None;
    }

    let now = SystemTime::now();
    let minutes = match now.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() / 60,
        Err(_) => 0,
    };

    // select task based on minute for
    // stateless stable rotation of displayed tasks
    Some(tasks[minutes as usize % tasks.len()].to_owned())
}
