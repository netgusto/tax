use crate::model::{Section, Task};
use crate::services::ContentGetter;
use regex::Regex;
use std::rc::Rc;
use std::time::SystemTime;

lazy_static! {
    static ref TASK_LINE_REGEX: Regex =
        Regex::new(r"(?m)^\s*(?:-|\*)\s+\[(x|\s*|>)\]\s+(.+?)$").unwrap();
    static ref TASK_NAME_FOCUSED_REGEX: Regex = Regex::new(r"(?m)\*\*.+\*\*").unwrap();
    static ref COMMENT_REGEX: Regex = Regex::new(r"(?m)^(.*?)[^:]//(.+?)$").unwrap();
    static ref HEADER_REGEX: Regex = Regex::new(r"(?m)^#{1,6}\s+(.*?)$").unwrap();
}

pub fn get_current_task(
    content_getter: &dyn ContentGetter,
    cycle: bool,
) -> Result<Option<(Task, bool)>, String> {
    let (focused_tasks, use_sections) = get_focused_open_tasks(content_getter)?;
    if focused_tasks.len() > 0 {
        return Ok(Some((focused_tasks[0].clone(), use_sections)));
    }

    let (tasks, _) = get_open_tasks(content_getter)?;
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
        return Ok(Some((
            tasks[minutes as usize % tasks.len()].clone(),
            use_sections,
        )));
    }

    Ok(Some((tasks[0].clone(), use_sections)))
}

pub fn get_all_tasks(content_getter: &dyn ContentGetter) -> Result<(Vec<Task>, bool), String> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut sections: Vec<Rc<Section>> = Vec::new();

    let mut section_num = 1;
    let mut task_num = 1;
    let mut line_num = 1;
    let mut current_section: Option<Rc<Section>> = None;

    for line in content_getter.get_contents()? {
        let l_str = line.as_str();

        match HEADER_REGEX.captures(l_str) {
            None => (),
            Some(cap) => {
                let section = Rc::from(Section {
                    name: cap[1].trim().to_string(),
                    num: section_num,
                    line: line.clone(),
                    line_num: line_num,
                });
                sections.push(section.clone());
                current_section = Some(section);

                section_num += 1;
                line_num += 1;
                continue;
            }
        };

        match TASK_LINE_REGEX.captures(l_str) {
            None => (),
            Some(cap) => {
                let check_symbol = cap[1].trim();
                let name = String::from(&cap[2]);
                let trimmed_name = name.trim();

                let (name_without_comment, comment) = text_get_comment(trimmed_name);

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
                    section: current_section.clone(),
                });

                task_num += 1;
            }
        }

        line_num += 1;
    }

    let use_section = sections.len() > 1;

    Ok((tasks, use_section))
}

pub fn get_open_tasks(content_getter: &dyn ContentGetter) -> Result<(Vec<Task>, bool), String> {
    match get_all_tasks(content_getter) {
        Ok((tasks, use_sections)) => Ok((
            tasks.into_iter().filter(|task| !task.is_checked).collect(),
            use_sections,
        )),
        Err(msg) => Err(msg),
    }
}

pub fn get_closed_tasks(content_getter: &dyn ContentGetter) -> Result<(Vec<Task>, bool), String> {
    match get_all_tasks(content_getter) {
        Ok((tasks, use_sections)) => Ok((
            tasks.into_iter().filter(|task| task.is_checked).collect(),
            use_sections,
        )),
        Err(msg) => Err(msg),
    }
}

pub fn get_focused_open_tasks(
    content_getter: &dyn ContentGetter,
) -> Result<(Vec<Task>, bool), String> {
    match get_open_tasks(content_getter) {
        Ok((tasks, use_sections)) => Ok((
            tasks.into_iter().filter(|task| task.is_focused).collect(),
            use_sections,
        )),
        Err(msg) => Err(msg),
    }
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
            content += format!("{}\n", added_line).as_str();
            added = true;
        }

        content += format!("{}\n", line).as_str();
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
    use crate::test_helpers::test::{get_std_test_contents, get_std_test_tasks, ContentGetterMock};

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
        match get_all_tasks(&ContentGetterMock::new(Ok(Vec::new()))) {
            Ok((tasks, _)) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_all_tasks(&ContentGetterMock::new(Ok(test_contents))) {
            Ok((tasks, _)) => assert_eq!(tasks, expected_tasks),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_get_open_tasks() {
        // Empty contents
        match get_open_tasks(&ContentGetterMock::new(Ok(Vec::new()))) {
            Ok((tasks, _)) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_open_tasks(&ContentGetterMock::new(Ok(test_contents))) {
            Ok((tasks, _)) => assert_eq!(
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
        match get_focused_open_tasks(&ContentGetterMock::new(Ok(Vec::new()))) {
            Ok((tasks, _)) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_focused_open_tasks(&ContentGetterMock::new(Ok(test_contents))) {
            Ok((tasks, _)) => assert_eq!(
                tasks,
                vec![expected_tasks[1].clone(), expected_tasks[5].clone()]
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_get_current_task() {
        // Empty contents
        match get_current_task(&ContentGetterMock::new(Ok(Vec::new())), false) {
            Ok(task) => assert_eq!(task, None),
            Err(e) => panic!(e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_current_task(&ContentGetterMock::new(Ok(test_contents)), false) {
            Ok(task) => assert_eq!(task, Some((expected_tasks[1].clone(), false))),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_text_add_line_in_contents() {
        match text_add_line_in_contents(
            &ContentGetterMock::new(Ok(Vec::new())),
            1,
            "Hello, World!".to_string(),
        ) {
            Ok(s) => assert_eq!(s, "Hello, World!\n"),
            Err(e) => panic!(e),
        }

        match text_add_line_in_contents(
            &ContentGetterMock::new(Ok(vec!["first line".to_string()])),
            1,
            "Hello, World!".to_string(),
        ) {
            Ok(s) => assert_eq!(s, "Hello, World!\nfirst line\n"),
            Err(e) => panic!(e),
        }

        match text_add_line_in_contents(
            &ContentGetterMock::new(Ok(vec!["first line".to_string()])),
            2,
            "Hello, World!".to_string(),
        ) {
            Ok(s) => assert_eq!(s, "first line\nHello, World!\n"),
            Err(e) => panic!(e),
        }

        match text_add_line_in_contents(
            &ContentGetterMock::new(Ok(vec!["first line".to_string(), "last line".to_string()])),
            2,
            "Hello, World!".to_string(),
        ) {
            Ok(s) => assert_eq!(s, "first line\nHello, World!\nlast line\n"),
            Err(e) => panic!(e),
        }

        match text_add_line_in_contents(
            &ContentGetterMock::new(Ok(vec![
                "# Header".to_string(),
                "".to_string(),
                "- [ ] Do the stuff".to_string(),
            ])),
            3,
            "Hello, World!".to_string(),
        ) {
            Ok(s) => assert_eq!(s, "# Header\n\nHello, World!\n- [ ] Do the stuff\n"),
            Err(e) => panic!(e),
        }
    }
}
