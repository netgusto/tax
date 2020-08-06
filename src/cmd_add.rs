use crate::cmd_list;
use crate::model::{Section, Task};
use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{
    filter_tasks_in_section, get_all_tasks, search_section, task_to_markdown, text_add_line_in_str,
    text_get_comment, text_is_focused, text_remove_focus, text_replace_line_in_str,
};

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    _user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    task_parts: Vec<String>,
    section_name: Option<String>,
    pos: AddPosition,
) -> Result<(), String> {
    let task_name = task_parts.join(" ");

    let (tasks, _use_sections, sections, focused_section) = get_all_tasks(content_getter)?;

    let (name_without_comment, comment) = text_get_comment(task_name.as_str());
    let is_task_focused = text_is_focused(name_without_comment.as_str());
    let plain_name = if is_task_focused {
        text_remove_focus(name_without_comment.as_str())
    } else {
        name_without_comment
    };

    let new_line = task_to_markdown(&Task {
        name: task_name.clone(),
        plain_name: plain_name.clone(),
        comment: comment,
        line: String::from(""),
        line_num: 0,
        is_checked: false,
        is_focused: is_task_focused,
        num: 0,
        section: None,
    });

    match section_name {
        None => (),
        Some(name) => match search_section(&name, &sections) {
            // Section has been specified
            None => return Err(format!("Section not found: {}", name)), // but does not exist;
            Some(section) => {
                return add_to_section(
                    tasks,
                    &section,
                    &new_line,
                    content_getter,
                    content_setter,
                    pos,
                );
            }
        },
    };

    match focused_section {
        None => (),
        Some(s) => {
            return add_to_section(
                tasks,
                s.as_ref(),
                &new_line,
                content_getter,
                content_setter,
                pos,
            );
        }
    };

    let (line_num, _task_num, _operation) = if tasks.len() == 0 {
        (1, 1, "APPEND".to_string())
    } else {
        match pos {
            AddPosition::Prepend => (tasks[0].line_num, 1, "PREPEND".to_string()),
            AddPosition::Append => (
                tasks[tasks.len() - 1].line_num + 1,
                tasks[tasks.len() - 1].num + 1,
                "APPEND".to_string(),
            ),
        }
    };

    let result = text_add_line_in_str(&content_getter.get_contents()?, line_num, &new_line);
    content_setter.set_contents(result)?;

    // FIXME: enable user_cmd_runner
    // match user_cmd_runner.build(
    //     String::from("add"),
    //     operation,
    //     format!("Added \"{}\"", task_name),
    // ) {
    //     Ok(Some(mut cmd)) => {
    //         user_cmd_runner.run(user_cmd_runner.env_single_task(new_task, &mut cmd))?;
    //     }
    //     Ok(None) => (),
    //     Err(e) => return Err(e),
    // };

    cmd_list::cmd(outputer, content_getter, task_formatter)
}

fn add_to_section(
    tasks: Vec<Task>,
    section: &Section,
    new_line: &str,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    pos: AddPosition,
) -> Result<(), String> {
    // Add task to section
    let section_tasks = filter_tasks_in_section(&tasks, section);

    if section_tasks.len() == 0 {
        // Section is empty;

        let new_lines = format!("\n{}", new_line);
        let result = text_add_line_in_str(
            &content_getter.get_contents()?,
            section.line_num_end + 1,
            &new_lines,
        );

        return content_setter.set_contents(result);
    } else {
        let (new_lines, line_num) = match pos {
            AddPosition::Prepend => (
                format!("{}\n{}", new_line, section_tasks[0].line),
                section_tasks[0].line_num,
            ),
            AddPosition::Append => (
                format!(
                    "{}\n{}",
                    section_tasks[section_tasks.len() - 1].line,
                    new_line,
                ),
                section_tasks[section_tasks.len() - 1].line_num,
            ),
        };

        let result =
            text_replace_line_in_str(&content_getter.get_contents()?, line_num, &new_lines);

        return content_setter.set_contents(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test::{
        ContentGetterMock, ContentSetterMock, StringOutputerMock, UserCmdRunnerMock,
    };

    #[test]
    fn test_cmd_add_to_empty_file() {
        let mut string_outputer = StringOutputerMock::new();
        let content_getter = ContentGetterMock::new(Ok("".to_string()));
        let mut content_setter = ContentSetterMock::new(Ok(()));
        let user_cmd_runner = UserCmdRunnerMock::new();
        let task_formatter = TaskFormatter::new(false);

        match cmd(
            &mut string_outputer,
            &content_getter,
            &mut content_setter,
            &user_cmd_runner,
            &task_formatter,
            vec!["**Some focused task** // with comments; see https://example.com".to_string()],
            None,
            AddPosition::Prepend,
        ) {
            Ok(()) => assert_eq!(
                content_setter.content,
                Some(String::from(
                    "- [ ] **Some focused task** // with comments; see https://example.com\n"
                ))
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cmd_add_to_top() {
        let mut string_outputer = StringOutputerMock::new();
        let content_getter = ContentGetterMock::new(Ok("- [ ] Existing task".to_string()));
        let mut content_setter = ContentSetterMock::new(Ok(()));
        let user_cmd_runner = UserCmdRunnerMock::new();
        let task_formatter = TaskFormatter::new(false);

        match cmd(
            &mut string_outputer,
            &content_getter,
            &mut content_setter,
            &user_cmd_runner,
            &task_formatter,
            vec!["Some task".to_string()],
            None,
            AddPosition::Prepend,
        ) {
            Ok(()) => assert_eq!(
                content_setter.content,
                Some(String::from("- [ ] Some task\n- [ ] Existing task\n"))
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cmd_add_to_bottom() {
        let mut string_outputer = StringOutputerMock::new();
        let content_getter = ContentGetterMock::new(Ok("- [ ] Existing task".to_string()));
        let mut content_setter = ContentSetterMock::new(Ok(()));
        let user_cmd_runner = UserCmdRunnerMock::new();
        let task_formatter = TaskFormatter::new(false);

        match cmd(
            &mut string_outputer,
            &content_getter,
            &mut content_setter,
            &user_cmd_runner,
            &task_formatter,
            vec!["Some task".to_string()],
            None,
            AddPosition::Append,
        ) {
            Ok(()) => assert_eq!(
                content_setter.content,
                Some(String::from("- [ ] Existing task\n- [ ] Some task\n"))
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cmd_add_top_section() {
        let mut string_outputer = StringOutputerMock::new();
        let content_getter = ContentGetterMock::new(Ok(vec![
            "# Section".to_string(),
            "".to_string(),
            "- [ ] Existing task".to_string(),
        ]
        .join("\n")));
        let mut content_setter = ContentSetterMock::new(Ok(()));
        let user_cmd_runner = UserCmdRunnerMock::new();
        let task_formatter = TaskFormatter::new(false);

        match cmd(
            &mut string_outputer,
            &content_getter,
            &mut content_setter,
            &user_cmd_runner,
            &task_formatter,
            vec!["Some task".to_string()],
            None,
            AddPosition::Prepend,
        ) {
            Ok(()) => assert_eq!(
                content_setter.content,
                Some(String::from(
                    "# Section\n\n- [ ] Some task\n- [ ] Existing task\n"
                ))
            ),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cmd_add_bottom_section() {
        let mut string_outputer = StringOutputerMock::new();
        let content_getter = ContentGetterMock::new(Ok(vec![
            "# Section".to_string(),
            "".to_string(),
            "- [ ] Existing task".to_string(),
        ]
        .join("\n")));
        let mut content_setter = ContentSetterMock::new(Ok(()));
        let user_cmd_runner = UserCmdRunnerMock::new();
        let task_formatter = TaskFormatter::new(false);

        match cmd(
            &mut string_outputer,
            &content_getter,
            &mut content_setter,
            &user_cmd_runner,
            &task_formatter,
            vec!["Some task".to_string()],
            None,
            AddPosition::Append,
        ) {
            Ok(()) => assert_eq!(
                content_setter.content,
                Some(String::from(
                    "# Section\n\n- [ ] Existing task\n- [ ] Some task\n"
                ))
            ),
            Err(e) => panic!(e),
        }
    }
}
