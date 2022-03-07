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
    static ref HEADER_REGEX: Regex = Regex::new(r"(?m)^(#{1,6})\s+(.*?)$").unwrap();
}

pub fn get_current_task(
    content_getter: &dyn ContentGetter,
    cycle: bool,
) -> Result<Option<(Task, bool)>, String> {
    let (open_tasks, use_sections, _, focused_section) = get_open_tasks(content_getter)?;

    let section_tasks = if use_sections && focused_section != None {
        open_tasks
            .into_iter()
            .filter(|t| filter_task_in_section_cbk(t, focused_section.clone().unwrap().as_ref()))
            .collect()
    } else {
        open_tasks
    };

    let focused_tasks = filter_focused_tasks(&section_tasks, true);

    if !focused_tasks.is_empty() {
        return Ok(Some((focused_tasks[0].clone(), use_sections)));
    }

    if section_tasks.is_empty() {
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
            section_tasks[minutes as usize % section_tasks.len()].clone(),
            use_sections,
        )));
    }

    Ok(Some((section_tasks[0].clone(), use_sections)))
}

type AllTasks = (Vec<Task>, bool, Vec<Rc<Section>>, Option<Rc<Section>>);

pub fn get_all_tasks(
    content_getter: &dyn ContentGetter,
) -> Result<AllTasks, String> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut sections: Vec<Rc<Section>> = Vec::new();

    let mut section_num = 1;
    let mut task_num = 1;
    let mut line_num = 1;
    let mut current_section: Option<Rc<Section>> = None;
    let mut focused_section: Option<Rc<Section>> = None;

    let mut building_section: Option<Rc<Section>> = None;

    for line in content_getter.get_contents()?.lines() {
        match HEADER_REGEX.captures(line) {
            None => (),
            Some(cap) => {
                match building_section {
                    None => (), // first section being built,
                    Some(s) => {
                        let mut section = s.as_ref().clone();
                        section.line_num_end = line_num - 1;

                        let section_rc = Rc::from(section);

                        if section_rc.is_focused && focused_section == None {
                            focused_section = Some(section_rc.clone());
                        }

                        sections.push(section_rc);
                        section_num += 1;
                    }
                }

                let header_markup = cap[1].trim();
                let section_name = cap[2].trim();
                let is_focused = text_is_focused(section_name);
                let plain_name = if is_focused {
                    text_remove_focus(section_name)
                } else {
                    section_name.to_string()
                };

                let section = Rc::from(Section {
                    name: section_name.to_string(),
                    plain_name,
                    is_focused,
                    num: section_num,
                    line: line.to_string(),
                    line_num,
                    line_num_end: 0,
                    level: header_markup.len(),
                });

                current_section = Some(section.clone());
                building_section = Some(section);

                line_num += 1;
                continue;
            }
        };

        match TASK_LINE_REGEX.captures(line) {
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
                    comment,
                    num: task_num,
                    is_checked: text_is_check_symbol(check_symbol),
                    line_num,
                    line: line.to_string(),
                    is_focused: is_task_focused,
                    section: current_section.clone(),
                });

                task_num += 1;
            }
        }

        line_num += 1;
    }

    match building_section {
        None => (),
        Some(s) => {
            let mut section = s.as_ref().clone();
            section.line_num_end = line_num - 1;

            let section_rc = Rc::from(section);

            if section_rc.is_focused && focused_section == None {
                focused_section = Some(section_rc.clone());
            }

            sections.push(section_rc);
        }
    }

    let use_section = sections.len() > 1;

    Ok((tasks, use_section, sections, focused_section))
}

pub fn get_open_tasks(
    content_getter: &dyn ContentGetter,
) -> Result<AllTasks, String> {
    match get_all_tasks(content_getter) {
        Ok((tasks, use_sections, sections, focused_section)) => Ok((
            filter_open_tasks(&tasks, true),
            use_sections,
            sections,
            focused_section,
        )),
        Err(msg) => Err(msg),
    }
}

pub fn get_closed_tasks(
    content_getter: &dyn ContentGetter,
) -> Result<AllTasks, String> {
    match get_all_tasks(content_getter) {
        Ok((tasks, use_sections, sections, focused_section)) => Ok((
            filter_open_tasks(&tasks, false),
            use_sections,
            sections,
            focused_section,
        )),
        Err(msg) => Err(msg),
    }
}

pub fn filter_open_tasks(tasks: &[Task], open: bool) -> Vec<Task> {
    tasks
        .iter()
        .filter(|t| filter_open_task_cbk(t, open)).cloned()
        .collect()
}

pub fn filter_focused_tasks(tasks: &[Task], focused: bool) -> Vec<Task> {
    tasks
        .iter()
        .filter(|t| filter_focused_task_cbk(t, focused)).cloned()
        .collect()
}

pub fn filter_tasks_in_section(tasks: &[Task], section: &Section) -> Vec<Task> {
    tasks
        .iter()
        .filter(|t| filter_task_in_section_cbk(t, section)).cloned()
        .collect()
}

pub fn filter_open_task_cbk(task: &Task, open: bool) -> bool {
    if open {
        !task.is_checked
    } else {
        task.is_checked
    }
}

pub fn filter_focused_task_cbk(task: &Task, focused: bool) -> bool {
    if focused {
        task.is_focused
    } else {
        !task.is_focused
    }
}

pub fn filter_task_in_section_cbk(task: &Task, section: &Section) -> bool {
    match &task.section {
        None => false,
        Some(s) => s.as_ref().num == section.num,
    }
}

pub fn text_is_check_symbol(s: &str) -> bool {
    s == "x"
}

pub fn section_to_markdown(section: &Section) -> String {
    format!(
        "{} {}",
        "#".repeat(section.level),
        if section.is_focused {
            text_add_focus(&section.plain_name)
        } else {
            section.plain_name.clone()
        },
    )
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

pub fn text_remove_lines_in_str(s: &str, line_nums: Vec<usize>) -> Result<String, String> {
    let mut line_num = 1;

    let mut content = String::from("");

    use std::collections::HashMap;
    let mut line_nums_hash = HashMap::new();

    for l in &line_nums {
        line_nums_hash.insert(*l, true);
    }

    for line in s.lines() {
        if !line_nums_hash.contains_key(&line_num) {
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    Ok(content)
}

pub fn text_replace_line_in_str(
    s: &str,
    replace_line_num: usize,
    replacement_line: &str,
) -> String {
    let mut line_num = 1;

    let mut content = String::from("");

    for line in s.lines() {
        if line_num == replace_line_num {
            content += format!("{}\n", replacement_line).as_str();
        } else {
            content += format!("{}\n", line).as_str();
        }

        line_num += 1;
    }

    content
}

pub fn text_add_line_in_str(s: &str, add_line_num: usize, added_line: &str) -> String {
    let mut line_num = 1;

    let mut content = String::from("");

    let mut added = false;

    for line in s.lines() {
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

    content
}

pub fn search_section(search: &str, sections: &[Rc<Section>]) -> Option<Section> {
    let section_name_lower = search.to_lowercase();

    let mut partial_match: Option<Section> = None;
    let mut exact_match: Option<Section> = None;

    for section_rc in sections {
        let section_cur = section_rc.as_ref();
        let section_cur_name_lower = section_cur.name.to_lowercase();

        if section_cur_name_lower == section_name_lower {
            exact_match = Some(section_cur.clone());
            break;
        } else if section_cur_name_lower.contains(&section_name_lower) {
            partial_match = Some(section_cur.clone());
        }
    }

    match exact_match {
        None => partial_match,
        _ => exact_match,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::test::{get_std_test_contents, get_std_test_tasks, ContentGetterMock};

    #[test]
    fn test_task_to_markdown() {
        let (expected_markdown, tasks) = get_std_test_tasks();
        let lines: Vec<&str> = expected_markdown.lines().collect();

        for (i, task) in tasks.into_iter().enumerate() {
            assert_eq!(lines[i], task_to_markdown(&task));
        }
    }

    #[test]
    fn test_is_check_symbol() {
        assert!(!text_is_check_symbol(""));
        assert!(!text_is_check_symbol("*"));
        assert!(text_is_check_symbol("x"));
        assert!(!text_is_check_symbol("X"));
    }

    #[test]
    fn test_get_all_tasks() {
        // Empty contents
        match get_all_tasks(&ContentGetterMock::new(Ok("".to_string()))) {
            Ok((tasks, _, _, _)) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!("{}", e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_all_tasks(&ContentGetterMock::new(Ok(test_contents))) {
            Ok((tasks, _, _, _)) => assert_eq!(tasks, expected_tasks),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_get_open_tasks() {
        // Empty contents
        match get_open_tasks(&ContentGetterMock::new(Ok("".to_string()))) {
            Ok((tasks, _, _, _)) => assert_eq!(tasks, Vec::new()),
            Err(e) => panic!("{}", e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_open_tasks(&ContentGetterMock::new(Ok(test_contents))) {
            Ok((tasks, _, _, _)) => assert_eq!(
                tasks,
                vec![
                    expected_tasks[0].clone(),
                    expected_tasks[1].clone(),
                    expected_tasks[4].clone(),
                    expected_tasks[5].clone(),
                ]
            ),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_get_current_task() {
        // Empty contents
        match get_current_task(&ContentGetterMock::new(Ok("".to_string())), false) {
            Ok(task) => assert_eq!(task, None),
            Err(e) => panic!("{}", e),
        }

        // Std contents
        let (test_contents, expected_tasks) = get_std_test_contents();

        match get_current_task(&ContentGetterMock::new(Ok(test_contents)), false) {
            Ok(task) => assert_eq!(task, Some((expected_tasks[1].clone(), false))),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_text_add_line_in_contents() {

        let s = text_add_line_in_str("", 1, "Hello, World!");
        assert_eq!(s, "Hello, World!\n");

        let s = text_add_line_in_str("first line", 1, "Hello, World!");
        assert_eq!(s, "Hello, World!\nfirst line\n");

        let s = text_add_line_in_str("first line", 2, "Hello, World!");
        assert_eq!(s, "first line\nHello, World!\n");

        let s = text_add_line_in_str("first line\nlast line", 2, "Hello, World!");
        assert_eq!(s, "first line\nHello, World!\nlast line\n");

        let s = text_add_line_in_str("# Header\n\n- [ ] Do the stuff", 3, "Hello, World!");
        assert_eq!(s, "# Header\n\nHello, World!\n- [ ] Do the stuff\n");
    }
}
