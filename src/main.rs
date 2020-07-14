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

struct ContentHandlerReal {
    path: String,
}

trait ContentGetter {
    fn get_contents(&self) -> Result<Vec<String>, String>;
}

impl ContentGetter for ContentHandlerReal {
    fn get_contents(&self) -> Result<Vec<String>, String> {
        match File::open(&self.path) {
            Err(_) => Err(format!("Could not open file {}", &self.path)),
            Ok(f) => {
                let reader = BufReader::new(f);
                let lines_result = reader.lines().collect::<Result<_, _>>();
                match lines_result {
                    Err(_) => Err(format!("Could not read file {}", &self.path)),
                    Ok(lines) => Ok(lines),
                }
            }
        }
    }
}

trait ContentSetter {
    fn set_contents(&self, contents: String) -> Result<(), String>;
}

impl ContentSetter for ContentHandlerReal {
    fn set_contents(&self, contents: String) -> Result<(), String> {
        match fs::write(&self.path, contents) {
            Ok(_) => Ok(()),
            Err(_) => Err(String::from("Unable to write file")),
        }
    }
}

trait StringOutputer {
    fn info(&mut self, s: String) -> ();
}

struct StringOutputerReal {}
impl StringOutputer for StringOutputerReal {
    fn info(&mut self, s: String) -> () {
        println!("{}", s);
    }
}

lazy_static! {
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref TASK_NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
}

#[derive(std::clone::Clone, Debug, PartialEq)]
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

    let file_path = taxfile_path_getter_real.get_taxfile_path()?;
    let content_handler_real = &ContentHandlerReal { path: file_path };

    let outputer = &mut StringOutputerReal {};

    match cmd {
        Some("edit") => cmd_edit(taxfile_path_getter_real),

        Some("focus") => cmd_focus(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            true,
        ),
        Some("blur") => cmd_focus(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            false,
        ),

        Some("check") => cmd_check(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            true,
        ),
        Some("uncheck") => cmd_check(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            false,
        ),

        Some("list") => cmd_list(outputer, content_handler_real),
        Some("current") => cmd_current(outputer, content_handler_real, false),
        Some("cycle") => cmd_current(outputer, content_handler_real, true),

        None => cmd_list(outputer, content_handler_real), // default: list
        _ => Err(format!("Unknown command \"{}\"", cmd.unwrap())),
    }
}

fn cmd_current(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    cycle: bool,
) -> Result<(), String> {
    match get_current_task(content_getter, cycle) {
        Ok(Some(task)) => outputer.info(format!("[{}] {}", task.num, task.name)),
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

fn cmd_list(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
) -> Result<(), String> {
    let tasks = get_open_tasks(content_getter)?;
    for task in tasks {
        outputer.info(format!("[{}] {}", task.num, task.name))
    }

    Ok(())
}

fn cmd_focus(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    args: Vec<String>,
    focus: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(outputer, content_getter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_completed {
        outputer.info(format!(
            "Completed, cannot proceed: [{}] {}",
            task.num, task.name
        ));
        return Ok(());
    }

    if focus && task.is_focused {
        outputer.info(format!("Already focused: [{}] {}", task.num, task.name));
        return Ok(());
    } else if !focus && !task.is_focused {
        outputer.info(format!("Already blured: [{}] {}", task.num, task.name));
        return Ok(());
    }

    let replacement_line = toggle_line_focus(task.line.clone(), focus);
    let action = if focus { "Focused" } else { "Blurred" };
    outputer.info(format!("{}: [{}] {}", action, task.num, task.name));

    let replaced_content =
        replace_line_in_contents(content_getter, task.line_num, replacement_line)?;

    content_setter.set_contents(replaced_content)
}

fn cmd_check(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    args: Vec<String>,
    completed: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(outputer, content_getter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if completed && task.is_completed {
        outputer.info(format!("Already checked: [{}] {}", task.num, task.name));
        return Ok(());
    } else if !completed && !task.is_completed {
        outputer.info(format!("Already unckecked: [{}] {}", task.num, task.name));
        return Ok(());
    }

    let checked_line = toggle_line_completion(task.line.clone(), completed);
    let action = if completed { "Checked" } else { "Unchecked" };
    outputer.info(format!("{}: [{}] {}", action, task.num, task.name));

    let replaced_content = replace_line_in_contents(content_getter, task.line_num, checked_line)?;

    content_setter.set_contents(replaced_content)
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

fn get_all_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<Task> = Vec::new();

    let mut task_num = 1;
    let mut line_num = 1;

    for line in content_getter.get_contents()? {
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

fn get_open_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_all_tasks(content_getter)?
        .into_iter()
        .filter(|task| !task.is_completed)
        .collect())
}

fn get_focused_open_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_open_tasks(content_getter)?
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
    content_getter: &dyn ContentGetter,
    cycle: bool,
) -> Result<Option<Task>, String> {
    let focused_tasks = get_focused_open_tasks(content_getter)?;
    if focused_tasks.len() > 0 {
        return Ok(Some(focused_tasks[0].clone()));
    }

    let tasks = get_open_tasks(content_getter)?;
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

fn replace_line_in_contents(
    content_getter: &dyn ContentGetter,
    replace_line_num: usize,
    replacement_line: String,
) -> Result<String, String> {
    let mut line_num = 1;

    let mut content = String::from("");

    for line in content_getter.get_contents()? {
        if line_num == replace_line_num {
            content += format!("{}\n", replacement_line).as_str();
        } else {
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    Ok(content)
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

    struct FileReaderMock {
        outcome: Result<Vec<String>, String>,
    }
    impl ContentGetter for FileReaderMock {
        fn get_contents(&self) -> Result<Vec<String>, String> {
            self.outcome.clone()
        }
    }

    struct StringOutputerMock {
        info_buf: Vec<String>,
    }
    impl StringOutputerMock {
        fn new() -> Self {
            return StringOutputerMock { info_buf: vec![] };
        }
        fn get_info(&self) -> String {
            self.info_buf.join("")
        }
    }
    impl StringOutputer for StringOutputerMock {
        fn info(&mut self, s: String) -> () {
            self.info_buf.push(String::from(format!("{}\n", s)));
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

    fn get_std_test_contents() -> (Vec<String>, Vec<Task>) {
        (
            vec![
                String::from("# Not a task"),
                String::from("- [ ] Standard unchecked"),
                String::from("- [] Collapsed unchecked"),
                String::from("- [ ] **Standard unchecked focused**"),
                String::from("* [ ] Star unchecked"),
                String::from("Also not a task"),
                String::from("- [x] Checked"),
                String::from("- [x] **Focused checked**"),
            ],
            vec![
                Task {
                    num: 1,
                    line_num: 2,
                    line: String::from("- [ ] Standard unchecked"),
                    name: String::from("Standard unchecked"),
                    is_completed: false,
                    is_focused: false,
                },
                Task {
                    num: 2,
                    line_num: 3,
                    line: String::from("- [] Collapsed unchecked"),
                    name: String::from("Collapsed unchecked"),
                    is_completed: false,
                    is_focused: false,
                },
                Task {
                    num: 3,
                    line_num: 4,
                    line: String::from("- [ ] **Standard unchecked focused**"),
                    name: String::from("**Standard unchecked focused**"),
                    is_completed: false,
                    is_focused: true,
                },
                Task {
                    num: 4,
                    line_num: 5,
                    line: String::from("* [ ] Star unchecked"),
                    name: String::from("Star unchecked"),
                    is_completed: false,
                    is_focused: false,
                },
                Task {
                    num: 5,
                    line_num: 7,
                    line: String::from("- [x] Checked"),
                    name: String::from("Checked"),
                    is_completed: true,
                    is_focused: false,
                },
                Task {
                    num: 6,
                    line_num: 8,
                    line: String::from("- [x] **Focused checked**"),
                    name: String::from("**Focused checked**"),
                    is_completed: true,
                    is_focused: true,
                },
            ],
        )
    }

    #[test]
    fn test_get_all_tasks() {
        // Empty contents
        match get_all_tasks(&FileReaderMock {
            outcome: Ok(Vec::new()),
        }) {
            Ok(tasks) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_all_tasks(&FileReaderMock {
            outcome: Ok(test_contents),
        }) {
            Ok(tasks) => assert_eq!(tasks, expected_tasks),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_get_open_tasks() {
        // Empty contents
        match get_open_tasks(&FileReaderMock {
            outcome: Ok(Vec::new()),
        }) {
            Ok(tasks) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_open_tasks(&FileReaderMock {
            outcome: Ok(test_contents),
        }) {
            Ok(tasks) => assert_eq!(
                tasks,
                vec![
                    expected_tasks[0].clone(),
                    expected_tasks[1].clone(),
                    expected_tasks[2].clone(),
                    expected_tasks[3].clone(),
                ]
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_get_focused_open_tasks() {
        // Empty contents
        match get_focused_open_tasks(&FileReaderMock {
            outcome: Ok(Vec::new()),
        }) {
            Ok(tasks) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_focused_open_tasks(&FileReaderMock {
            outcome: Ok(test_contents),
        }) {
            Ok(tasks) => assert_eq!(tasks, vec![expected_tasks[2].clone()]),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_get_current_task() {
        // Empty contents
        match get_current_task(
            &FileReaderMock {
                outcome: Ok(Vec::new()),
            },
            false,
        ) {
            Ok(task) => assert_eq!(task, None),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_current_task(
            &FileReaderMock {
                outcome: Ok(test_contents),
            },
            false,
        ) {
            Ok(task) => assert_eq!(task, Some(expected_tasks[2].clone())),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cmd_current() {
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd_current(outputer_mock, content_getter_mock, false).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd_current(outputer_mock, content_getter_mock, false).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[3] **Standard unchecked focused**\n"
            );
        }
    }

    #[test]
    fn test_cmd_list() {
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd_list(outputer_mock, content_getter_mock).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd_list(outputer_mock, content_getter_mock).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[1] Standard unchecked\n[2] Collapsed unchecked\n[3] **Standard unchecked focused**\n[4] Star unchecked\n"
            );
        }
    }
}
