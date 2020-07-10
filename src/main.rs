#[macro_use]
extern crate lazy_static;
extern crate dirs;

use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use std::io::{prelude::*, BufReader};

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(\*|\s*)\]\s+(.+?)$").unwrap();
}

#[derive(std::clone::Clone)]
pub struct Task {
    num: usize,
    name: String,
    is_completed: bool,
    line_num: usize,
    line: String,
}

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
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() > 0 {
        let cmd = args[0].as_str();
        if cmd == "edit" {
            return cmd_edit();
        }

        if cmd == "check" {
            args.remove(0);
            return cmd_check(args);
        }

        if cmd == "list" {
            return cmd_list();
        }

        if cmd.chars().nth(0).unwrap_or_default() != '-' {
            // not a flag for display
            return Err(format!("Unknown command \"{}\"", cmd));
        }
    }

    cmd_display(args)
}

fn cmd_display(args: Vec<String>) -> Result<(), String> {
    let mut rotate = true;
    for arg in args {
        if arg == "--first" || arg == "-1" {
            rotate = false
        }
    }

    let tasks_result = open_tasks();
    if !tasks_result.is_ok() {
        // display nothing
        return Ok(());
    }

    match pick_task(tasks_result.unwrap(), rotate) {
        Some(task) => println!("[{}] {}", task.num, task.name),
        None => (),
    }

    Ok(())
}

fn cmd_edit() -> Result<(), String> {
    let str_file_path = get_file_path().unwrap();

    let res = env::var("EDITOR");
    if !res.is_ok() {
        return Err(String::from(
            "Please set $EDITOR in environment to use \"edit\".",
        ));
    }

    let editor = res.unwrap();

    if let Ok(mut child) = Command::new(editor).arg(str_file_path).spawn() {
        match child.wait() {
            Ok(_) => (),
            Err(_) => {
                return Err(String::from("Could not run $EDITOR"));
            }
        }
    } else {
        return Err(String::from("Could not run $EDITOR"));
    }

    Ok(())
}

fn cmd_list() -> Result<(), String> {
    let tasks = open_tasks()?;
    for task in tasks {
        println!("[{}] {}", task.num, task.name)
    }

    Ok(())
}

fn cmd_check(args: Vec<String>) -> Result<(), String> {
    // Read file line by line
    // If line matches pattern =>
    //   If rank matches => check
    // => display the checked task

    if args.len() == 0 {
        return cmd_list();
    }

    let rank_one_based = str::parse::<usize>(args[0].as_str()).unwrap();

    if rank_one_based == 0 {
        return cmd_list();
    }

    let tasks = all_tasks()?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_completed {
        println!("Already completed: [{}] {}", task.num, task.name)
    } else {
        let checked_line = check_line(task.line.clone());
        println!("Completed: [{}] {}", task.num, task.name);

        let str_file_path = get_file_path()?;

        let file_result = File::open(&str_file_path);
        if !file_result.is_ok() {
            return Err(String::from("Cannot open taxfile"));
        }
        let reader = BufReader::new(file_result.unwrap());

        let mut line_num = 1;

        let mut content = String::from("");

        for line_result in reader.lines() {
            if line_num == task.line_num {
                content += format!("{}\n", checked_line).as_str();
            } else {
                let line = line_result.unwrap();
                content += format!("{}\n", line).as_str();
            }

            line_num += 1;
        }

        fs::write(str_file_path, content).expect("Unable to write file");
    }

    Ok(())
}

fn home_dir() -> Result<PathBuf, String> {
    match dirs::home_dir() {
        None => return Err(String::from("Cannot find home dir")),
        Some(h) => Ok(h),
    }
}

fn get_file_path() -> Result<String, String> {
    let file_path: String;

    let res = env::var("TAXFILE");
    if res.is_ok() {
        file_path = res.unwrap();
    } else {
        let home = match home_dir() {
            Err(e) => {
                return Err(e);
            }
            Ok(home) => home,
        };

        file_path = String::from(home.join(Path::new("taxfile")).to_str().unwrap());
    }

    Ok(file_path)
}

fn pick_task(tasks: Vec<Task>, rotate: bool) -> Option<Task> {
    if tasks.len() == 0 {
        return None;
    }

    if rotate {
        let now = SystemTime::now();
        let minutes = match now.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs() / 60,
            Err(_) => 0,
        };

        // select task based on minute for
        // stateless stable rotation of displayed tasks
        return Some(tasks[minutes as usize % tasks.len()].clone());
    }

    Some(tasks[0].clone())
}

fn all_tasks() -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();

    let str_file_path = get_file_path()?;

    let file_result = File::open(str_file_path);
    if !file_result.is_ok() {
        return Err(String::from("Cannot open taxfile"));
    }
    let reader = BufReader::new(file_result.unwrap());

    let mut task_num = 1;
    let mut line_num = 1;

    for line_result in reader.lines() {
        let line = line_result.unwrap();
        match RE.captures(line.as_str()) {
            None => (),
            Some(cap) => {
                tasks.push(Task {
                    name: String::from(&cap[2]),
                    num: task_num,
                    is_completed: cap[1].trim() == "*",
                    line_num: line_num,
                    line: String::from(&cap[0]),
                });

                task_num += 1;
            }
        }

        line_num += 1;
    }

    Ok(tasks)
}

fn open_tasks() -> Result<Vec<Task>, String> {
    Ok(all_tasks()?
        .into_iter()
        .filter(|task| !task.is_completed)
        .collect())
}

fn check_line(line: String) -> String {
    let re = Regex::new(
        r"(?x)
        ^(?P<prefix>\s*(?:-|\*)\s+)
        (?P<checkbox>\[\s*])
        (?P<suffix>.+?)
        $
    ",
    )
    .unwrap();

    re.replace_all(&line, "$prefix[*]$suffix").into()
}
