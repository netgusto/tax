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

mod tax;

type EnvGetter = fn(&str) -> Option<String>;
fn env_getter_real(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

type HomeGetter = fn() -> Option<PathBuf>;
fn home_getter_real() -> Option<PathBuf> {
    match dirs::home_dir() {
        None => return None,
        Some(h) => Some(h),
    }
}

trait TaxfilePathGetter {
    fn get_taxfile_path(&self) -> Result<String, String>;
}

struct TaxfilePathGetterReal {
    get_env: EnvGetter,
    get_home: HomeGetter,
}

impl TaxfilePathGetter for TaxfilePathGetterReal {
    fn get_taxfile_path(&self) -> Result<String, String> {
        match (self.get_env)("TAXFILE") {
            Some(v) => Ok(v),
            None => match (self.get_home)() {
                None => Err(String::from("Could not find home dir")),
                Some(home) => Ok(String::from(
                    home.join(Path::new("taxfile")).to_str().unwrap(),
                )),
            },
        }
    }
}

lazy_static! {
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref TASK_NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
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

fn main() -> Result<(), String> {
    run_app(env::args().collect())
}

fn run_app(args: Vec<String>) -> Result<(), String> {
    let cmd: Option<&str> = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    let taxfile_path_getter_real = &TaxfilePathGetterReal {
        get_env: env_getter_real,
        get_home: home_getter_real,
    };

    match cmd {
        Some("edit") => cmd_edit(taxfile_path_getter_real),

        Some("focus") => cmd_focus(taxfile_path_getter_real, args, true),
        Some("blur") => cmd_focus(taxfile_path_getter_real, args, false),

        Some("check") => cmd_check(taxfile_path_getter_real, args, true),
        Some("uncheck") => cmd_check(taxfile_path_getter_real, args, false),

        Some("list") => cmd_list(taxfile_path_getter_real),
        Some("current") => cmd_current(taxfile_path_getter_real, false),
        Some("cycle") => cmd_current(taxfile_path_getter_real, true),

        None => cmd_list(taxfile_path_getter_real), // default: list
        _ => Err(format!("Unknown command \"{}\"", cmd.unwrap())),
    }
}

fn cmd_current(taxfile_path_getter: &dyn TaxfilePathGetter, cycle: bool) -> Result<(), String> {
    match get_current_task(taxfile_path_getter, cycle) {
        Ok(Some(task)) => println!("[{}] {}", task.num, task.name),
        _ => (),
    };
    Ok(())
}

fn cmd_edit(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<(), String> {
    let str_file_path = taxfile_path_getter.get_taxfile_path().unwrap();

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

fn cmd_list(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<(), String> {
    let tasks = get_open_tasks(taxfile_path_getter)?;
    for task in tasks {
        println!("[{}] {}", task.num, task.name)
    }

    Ok(())
}

fn cmd_focus(
    taxfile_path_getter: &dyn TaxfilePathGetter,
    args: Vec<String>,
    focus: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(taxfile_path_getter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(taxfile_path_getter)?;
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

    replace_line_in_file(taxfile_path_getter, task.line_num, replacement_line)
}

fn cmd_check(
    taxfile_path_getter: &dyn TaxfilePathGetter,
    args: Vec<String>,
    completed: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(taxfile_path_getter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(taxfile_path_getter)?;
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

    replace_line_in_file(taxfile_path_getter, task.line_num, checked_line)
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

fn get_all_tasks(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();
    let str_file_path = taxfile_path_getter.get_taxfile_path()?;

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

                let is_focused = name.len() > 4 && TASK_NAME_FOCUSED_REGEX.is_match(&name);
                tasks.push(Task {
                    name: name,
                    num: task_num,
                    is_completed: is_check_symbol(check_symbol),
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

fn get_open_tasks(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<Vec<Task>, String> {
    Ok(get_all_tasks(taxfile_path_getter)?
        .into_iter()
        .filter(|task| !task.is_completed)
        .collect())
}

fn get_focused_tasks(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<Vec<Task>, String> {
    Ok(get_open_tasks(taxfile_path_getter)?
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
        None => line,
        Some(cap) => {
            let name = String::from(&cap[4]);

            let is_focused = TASK_NAME_FOCUSED_REGEX.is_match(&name);
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

fn get_current_task(
    taxfile_path_getter: &dyn TaxfilePathGetter,
    cycle: bool,
) -> Result<Option<Task>, String> {
    let focused_tasks = get_focused_tasks(taxfile_path_getter)?;
    if focused_tasks.len() > 0 {
        return Ok(Some(focused_tasks[0].clone()));
    }

    let tasks = get_open_tasks(taxfile_path_getter)?;
    if tasks.len() == 0 {
        return Ok(None);
    }

    if cycle {
        let now = SystemTime::now();
        let minutes = match now.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs() / 60,
            Err(_) => 0,
        };

        // select task based on minute for
        // stateless stable rotation of displayed tasks
        return Ok(Some(tasks[minutes as usize % tasks.len()].clone()));
    }

    Ok(Some(tasks[0].clone()))
}

fn replace_line_in_file(
    taxfile_path_getter: &dyn TaxfilePathGetter,
    replace_line_num: usize,
    replacement_line: String,
) -> Result<(), String> {
    let str_file_path = taxfile_path_getter.get_taxfile_path()?;

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

fn is_check_symbol(s: &str) -> bool {
    return s == "x";
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_getter_none(_: &str) -> Option<String> {
        return None;
    }

    fn home_getter_guybrush() -> Option<PathBuf> {
        return Some(PathBuf::from("/home/guybrush"));
    }

    fn env_getter_taxfile(name: &str) -> Option<String> {
        match name {
            "TAXFILE" => Some("/path/to/overriden/taxfile".to_string()),
            _ => None,
        }
    }

    #[test]
    fn test_is_check_symbol() {
        assert_eq!(is_check_symbol(""), false);
        assert_eq!(is_check_symbol("*"), false);
        assert_eq!(is_check_symbol("x"), true);
        assert_eq!(is_check_symbol("X"), false);
    }

    #[test]
    fn test_taxfile_path_getter_real() {
        let path_getter_noenv = &TaxfilePathGetterReal {
            get_env: env_getter_none,
            get_home: home_getter_guybrush,
        };
        assert_eq!(
            path_getter_noenv.get_taxfile_path(),
            Ok(String::from("/home/guybrush/taxfile"))
        );

        let path_getter_yesenv = &TaxfilePathGetterReal {
            get_env: env_getter_taxfile,
            get_home: home_getter_guybrush,
        };

        assert_eq!(
            path_getter_yesenv.get_taxfile_path(),
            Ok(String::from("/path/to/overriden/taxfile"))
        );
    }

    #[test]
    fn test_line_focus() {
        assert_eq!(
            toggle_line_focus(String::from("This is not a task line"), true),
            String::from("This is not a task line")
        );

        assert_eq!(
            toggle_line_focus(String::from("* [] This is a task"), true),
            String::from("* [] **This is a task**")
        );

        assert_eq!(
            toggle_line_focus(String::from("- [ ] This is a task"), true),
            String::from("- [ ] **This is a task**")
        );

        assert_eq!(
            toggle_line_focus(String::from("- [x] **This is a task**"), true),
            String::from("- [x] **This is a task**")
        );
    }

    #[test]
    fn test_line_blur() {
        assert_eq!(
            toggle_line_focus(String::from("This is not a task line"), false),
            String::from("This is not a task line")
        );

        assert_eq!(
            toggle_line_focus(String::from("* [] This is a task"), false),
            String::from("* [] This is a task")
        );

        assert_eq!(
            toggle_line_focus(String::from("- [x] **This is a task**"), false),
            String::from("- [x] This is a task")
        );
    }

    #[test]
    fn test_line_check() {
        assert_eq!(
            toggle_line_completion(String::from("This is not a task line"), true),
            String::from("This is not a task line")
        );

        assert_eq!(
            toggle_line_completion(String::from("* [] This is a task"), true),
            String::from("* [x] This is a task")
        );

        assert_eq!(
            toggle_line_completion(String::from("- [ ] **This is a task**"), true),
            String::from("- [x] **This is a task**")
        );
    }

    #[test]
    fn test_line_uncheck() {
        assert_eq!(
            toggle_line_completion(String::from("This is not a task line"), false),
            String::from("This is not a task line")
        );

        assert_eq!(
            toggle_line_completion(String::from("* [] This is a task"), false),
            String::from("* [] This is a task")
        );

        assert_eq!(
            toggle_line_completion(String::from("- [x] **This is a task**"), false),
            String::from("- [ ] **This is a task**")
        );
    }
}
