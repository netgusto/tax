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
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
}

#[derive(std::clone::Clone)]
pub struct Task {
    num: usize,
    name: String,
    is_completed: bool,
    line_num: usize,
    line: String,
    is_focused: bool,
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
    let args: Vec<String> = env::args().collect();
    let cmd: Option<&str> = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    match cmd {
        Some("edit") => cmd_edit(),

        Some("focus") => cmd_focus(args, true),
        Some("blur") => cmd_focus(args, false),

        Some("check") => cmd_check(args, true),
        Some("uncheck") => cmd_check(args, false),

        Some("list") => cmd_list(),
        Some("current") => cmd_current(false),
        Some("cycle") => cmd_current(true),

        None => cmd_list(), // default: list
        _ => Err(format!("Unknown command \"{}\"", cmd.unwrap())),
    }
}

fn cmd_current(cycle: bool) -> Result<(), String> {
    match current_task(cycle) {
        Ok(Some(task)) => println!("[{}] {}", task.num, task.name),
        _ => (),
    };
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

fn cmd_focus(args: Vec<String>, focus: bool) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(),
        Some(rank) => rank,
    };

    let tasks = all_tasks()?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_completed {
        println!("Completed, cannot proceed: [{}] {}", task.num, task.name);
        return Ok(());
    }

    if focus && task.is_focused {
        println!("Already focused: [{}] {}", task.num, task.name);
        return Ok(());
    } else if !focus && !task.is_focused {
        println!("Already blured: [{}] {}", task.num, task.name);
        return Ok(());
    }

    let replacement_line = toggle_line_focus(task.line.clone(), focus);
    let action = if focus { "Focused" } else { "Blurred" };
    println!("{}: [{}] {}", action, task.num, task.name);

    replace_line_in_file(task.line_num, replacement_line)
}

fn cmd_check(args: Vec<String>, completed: bool) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(),
        Some(rank) => rank,
    };

    let tasks = all_tasks()?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if completed && task.is_completed {
        println!("Already checked: [{}] {}", task.num, task.name);
        return Ok(());
    } else if !completed && !task.is_completed {
        println!("Already unckecked: [{}] {}", task.num, task.name);
        return Ok(());
    }

    let checked_line = toggle_line_completion(task.line.clone(), completed);
    let action = if completed { "Checked" } else { "Unchecked" };
    println!("{}: [{}] {}", action, task.num, task.name);

    replace_line_in_file(task.line_num, checked_line)
}

fn get_cmd_rank_arg(args: Vec<String>) -> Result<Option<usize>, String> {
    if args.len() == 2 {
        return Ok(None);
    }

    let rank_one_based = match str::parse::<usize>(args[2].as_str()) {
        Ok(0) | Err(_) => return Err(format!("Invalid task rank \"{}\"", args[2])),
        Ok(v) => v,
    };

    return Ok(Some(rank_one_based));
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
        match TASK_LINE_REGEX.captures(line.as_str()) {
            None => (),
            Some(cap) => {
                let check_symbol = cap[1].trim();
                let name = String::from(&cap[2]);

                let is_focused = name.len() > 4 && NAME_FOCUSED_REGEX.is_match(&name);
                tasks.push(Task {
                    name: name,
                    num: task_num,
                    is_completed: check_symbol == "x",
                    line_num: line_num,
                    line: String::from(&cap[0]),
                    is_focused: is_focused,
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

fn doing_tasks() -> Result<Vec<Task>, String> {
    Ok(open_tasks()?
        .into_iter()
        .filter(|task| task.is_focused)
        .collect())
}

fn toggle_line_completion(line: String, completed: bool) -> String {
    let (re, replacement) = if completed {
        (
            Regex::new(
                r"(?x)
                    ^(?P<prefix>\s*(?:-|\*)\s+)
                    (?P<checkbox>\[\s*])
                    (?P<suffix>.+?)
                    $
                ",
            )
            .unwrap(),
            "$prefix[x]$suffix",
        )
    } else {
        (
            Regex::new(
                r"(?x)
                    ^(?P<prefix>\s*(?:-|\*)\s+)
                    (?P<checkbox>\[x])
                    (?P<suffix>.+?)
                    $
                ",
            )
            .unwrap(),
            "$prefix[ ]$suffix",
        )
    };

    re.replace_all(&line, replacement).into()
}

fn toggle_line_focus(line: String, focused: bool) -> String {
    let re = Regex::new(
        r"(?x)
            ^(?P<prefix>\s*(?:-|\*)\s+)
            (?P<checkbox>\[(?:\s*|x)])
            (?P<spacing>\s+)
            (?P<suffix>.+?)
            $
        ",
    )
    .unwrap();

    return match re.captures(line.as_str()) {
        None => String::from("no1"),
        Some(cap) => {
            let name = String::from(&cap[4]);

            let is_focused = NAME_FOCUSED_REGEX.is_match(&name);
            if is_focused && focused {
                return line;
            } else if !is_focused && !focused {
                return line;
            }

            let replacement = if focused {
                format!("**{}**", name)
            } else {
                name.chars().take(name.len() - 2).skip(2).collect()
            };

            format!("{}{}{}{}", &cap[1], &cap[2], &cap[3], replacement)
        }
    };
}

fn current_task(cycle: bool) -> Result<Option<Task>, String> {
    let tasks_result = open_tasks();
    if !tasks_result.is_ok() {
        // display nothing
        return Ok(None);
    }
    let doing = doing_tasks()?;
    if doing.len() > 0 {
        return Ok(Some(doing[0].clone()));
    }

    match pick_task(tasks_result.unwrap(), cycle) {
        Some(task) => Ok(Some(task)),
        None => Ok(None),
    }
}

fn replace_line_in_file(replace_line_num: usize, replacement_line: String) -> Result<(), String> {
    let str_file_path = get_file_path()?;

    let file_result = File::open(&str_file_path);
    if !file_result.is_ok() {
        return Err(String::from("Cannot open taxfile"));
    }
    let reader = BufReader::new(file_result.unwrap());

    let mut line_num = 1;

    let mut content = String::from("");

    for line_result in reader.lines() {
        if line_num == replace_line_num {
            content += format!("{}\n", replacement_line).as_str();
        } else {
            let line = line_result.unwrap();
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    fs::write(str_file_path, content).expect("Unable to write file");

    Ok(())
}
