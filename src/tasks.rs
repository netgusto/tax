use crate::model::Task;
use crate::services::ContentGetter;
use regex::Regex;
use std::time::SystemTime;

lazy_static! {
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref TASK_NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
    static ref COMMENT_REGEX: Regex = Regex::new(r"(?m)^(.*?)[^:]//(.+?)$").unwrap();
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

                let (name_without_comment, comment) = text_get_comment(name.as_str());

                let is_task_focused = text_is_focused(name_without_comment.as_str());
                tasks.push(Task {
                    name: name_without_comment.clone(),
                    plain_name: if is_task_focused {
                        text_remove_focus(name_without_comment.as_str())
                    } else {
                        name_without_comment
                    },
                    comment: comment,
                    num: task_num,
                    is_checked: text_is_check_symbol(check_symbol),
                    line_num: line_num,
                    line: String::from(&cap[0]),
                    is_focused: is_task_focused,
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
        .filter(|task| !task.is_checked)
        .collect())
}

pub fn get_closed_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_all_tasks(content_getter)?
        .into_iter()
        .filter(|task| task.is_checked)
        .collect())
}

pub fn get_focused_open_tasks(content_getter: &dyn ContentGetter) -> Result<Vec<Task>, String> {
    Ok(get_open_tasks(content_getter)?
        .into_iter()
        .filter(|task| task.is_focused)
        .collect())
}

pub fn text_is_check_symbol(s: &str) -> bool {
    return s == "x";
}

pub fn task_to_markdown(task: &Task) -> String {
    format!(
        "- [{}] {}{}",
        if task.is_checked { "x" } else { " " },
        if task.is_focused {
            text_add_focus(&task.plain_name)
        } else {
            task.plain_name.clone()
        },
        if task.comment != None {
            format!(" // {}", task.comment.clone().unwrap())
        } else {
            String::from("")
        }
    )
}

pub fn text_add_focus(name: &str) -> String {
    format!("**{}**", name)
}

pub fn text_remove_focus(name: &str) -> String {
    name.chars().take(name.len() - 2).skip(2).collect()
}

pub fn text_is_focused(task_name: &str) -> bool {
    task_name.len() > 4 && TASK_NAME_FOCUSED_REGEX.is_match(task_name)
}

pub fn text_get_comment(task_name: &str) -> (String, Option<String>) {
    match COMMENT_REGEX.captures(task_name) {
        None => (String::from(task_name), None),
        Some(cap2) => (cap2[1].trim().to_string(), Some(cap2[2].trim().to_string())),
    }
}

pub fn text_remove_lines_in_contents(
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

pub fn text_replace_line_in_contents(
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

pub fn text_add_line_in_contents(
    content_getter: &dyn ContentGetter,
    add_line_num: usize,
    added_line: String,
) -> Result<String, String> {
    let mut line_num = 1;

    let mut content = String::from("");

    let mut added = false;

    for line in content_getter.get_contents()? {
        if line_num == add_line_num {
            content += format!("{}\n{}\n", added_line, line).as_str();
            added = true;
        } else {
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    if !added {
        // either empty file or line past end of file
        content += format!("{}\n", added_line).as_str();
    }

    Ok(content)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::{get_std_test_contents, get_std_test_tasks, FileReaderMock};

    #[test]
    fn test_task_to_markdown() {
        let (expected_markdown, tasks) = get_std_test_tasks();

        let mut i = 0;
        for task in tasks {
            assert_eq!(expected_markdown[i].clone(), task_to_markdown(&task));
            i += 1;
        }
    }

    #[test]
    fn test_is_check_symbol() {
        assert_eq!(text_is_check_symbol(""), false);
        assert_eq!(text_is_check_symbol("*"), false);
        assert_eq!(text_is_check_symbol("x"), true);
        assert_eq!(text_is_check_symbol("X"), false);
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
                    expected_tasks[4].clone(),
                    expected_tasks[5].clone(),
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
            Ok(tasks) => assert_eq!(
                tasks,
                vec![expected_tasks[1].clone(), expected_tasks[5].clone()]
            ),
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
            Ok(task) => assert_eq!(task, Some(expected_tasks[1].clone())),
            Err(e) => panic!(e),
        }
    }
}
