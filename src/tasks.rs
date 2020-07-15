use crate::model::Task;
use crate::services::ContentGetter;
use regex::Regex;
use std::time::SystemTime;

lazy_static! {
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref TASK_NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
}

pub fn get_current_task(
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

pub fn get_all_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
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

pub fn get_open_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_all_tasks(content_getter)?
        .into_iter()
        .filter(|task| !task.is_completed)
        .collect())
}

pub fn get_closed_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_all_tasks(content_getter)?
        .into_iter()
        .filter(|task| task.is_completed)
        .collect())
}

pub fn get_focused_open_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_open_tasks(content_getter)?
        .into_iter()
        .filter(|task| task.is_focused)
        .collect())
}

pub fn is_check_symbol(s: &str) -> bool {
    return s == "x";
}

pub fn format_numbered_task(task: &Task) -> String {
    format!("[{}] {}", task.num, task.name)
}

pub fn toggle_line_completion(line: String, completed: bool) -> String {
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

pub fn toggle_line_focus(line: String, focused: bool) -> String {
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

    match re.captures(line.as_str()) {
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
    }
}

pub fn remove_lines_in_contents(
    content_getter: &dyn ContentGetter,
    line_nums: Vec<usize>,
) -> Result<String, String> {
    let mut line_num = 1;

    let mut content = String::from("");

    use std::collections::HashMap;
    let mut line_nums_hash = HashMap::new();

    for l in &line_nums {
        line_nums_hash.insert(*l, true);
    }

    for line in content_getter.get_contents()? {
        if !line_nums_hash.contains_key(&line_num) {
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    Ok(content)
}

pub fn replace_line_in_contents(
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::{get_std_test_contents, FileReaderMock};

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

    #[test]
    fn test_is_check_symbol() {
        assert_eq!(is_check_symbol(""), false);
        assert_eq!(is_check_symbol("*"), false);
        assert_eq!(is_check_symbol("x"), true);
        assert_eq!(is_check_symbol("X"), false);
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
}
