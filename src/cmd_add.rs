use crate::cmd_list;
use crate::model::{Section, Task};
use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{
    filter_tasks_in_section, get_all_tasks, search_section, task_to_markdown, text_add_line_in_str,
    text_get_comment, text_is_focused, text_remove_focus, text_replace_line_in_str,
};

use std::rc::Rc;

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
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

    let mut new_task = Task {
        name: task_name.clone(),
        plain_name: plain_name.clone(),
        comment,
        line: String::from(""),
        line_num: 0,
        is_checked: false,
        is_focused: is_task_focused,
        num: 0,
        section: None,
    };

    new_task.line = task_to_markdown(&new_task);
    let mut added: bool = false;
    let mut display_all: bool = false;

    match section_name {
        None => (),
        Some(name) => match search_section(&name, &sections) {
            // Section has been specified
            None => return Err(format!("Section not found: {}", name)), // but does not exist;
            Some(section) => {
                match add_to_section(
                    &tasks,
                    &sections,
                    &section,
                    &new_task.line,
                    content_getter,
                    content_setter,
                    &pos,
                ) {
                    Err(e) => return Err(e),
                    Ok((line_num, task_num)) => {
                        added = true;
                        new_task.line_num = line_num;
                        new_task.num = task_num;

                        // display all tasks if task added to section not focused
                        display_all = match &focused_section {
                            None => false,
                            Some(f) => f.num != section.num,
                        };
                    }
                };
            }
        },
    };

    if !added {
        match focused_section {
            None => (),
            Some(s) => {
                match add_to_section(
                    &tasks,
                    &sections,
                    s.as_ref(),
                    &new_task.line,
                    content_getter,
                    content_setter,
                    &pos,
                ) {
                    Err(e) => return Err(e),
                    Ok((line_num, task_num)) => {
                        added = true;
                        new_task.line_num = line_num;
                        new_task.num = task_num;
                    }
                }
            }
        };
    }

    if !added {
        let (line_num, task_num) = if tasks.is_empty() {
            (1, 1)
        } else {
            match pos {
                AddPosition::Prepend => (tasks[0].line_num, 1),
                AddPosition::Append => (
                    tasks[tasks.len() - 1].line_num + 1,
                    tasks[tasks.len() - 1].num + 1,
                ),
            }
        };

        let new_content =
            text_add_line_in_str(&content_getter.get_contents()?, line_num, &new_task.line);
        content_setter.set_contents(new_content)?;

        new_task.line_num = line_num;
        new_task.num = task_num;
    }

    if let Err(e) = call_user_cmd_runner(
        user_cmd_runner,
        match pos {
            AddPosition::Prepend => "PREPEND",
            AddPosition::Append => "APPEND",
        },
        &new_task,
    ) {
        return Err(e)
    }

    cmd_list::cmd(outputer, content_getter, task_formatter, display_all) // FIXME: all or not depending of whether the task is in displayed section
}

fn call_user_cmd_runner(
    user_cmd_runner: &dyn UserCmdRunner,
    operation: &str,
    task: &Task,
) -> Result<(), String> {
    match user_cmd_runner.build("add", operation, &format!("Added \"{}\"", task.name)) {
        Ok(Some(mut cmd)) => user_cmd_runner.run(user_cmd_runner.env_single_task(task, &mut cmd)),
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

fn add_to_section(
    tasks: &[Task],
    sections: &[Rc<Section>],
    section: &Section,
    new_line: &str,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    pos: &AddPosition,
) -> Result<(usize, usize), String> {
    // Add task to section
    let section_tasks = filter_tasks_in_section(tasks, section);

    let content = match content_getter.get_contents() {
        Err(e) => return Err(e),
        Ok(content) => content,
    };

    let line_num: usize;
    let task_num: usize;
    let new_lines: String;
    let new_content: String;

    if section_tasks.is_empty() {
        task_num = if section.num == 1 {
            1
        } else {
            // Find previous task num!
            let mut cur_section_num = section.num - 1;
            let mut found: Option<usize> = None;

            while cur_section_num >= 1 {
                let tasks_in_cur_section =
                    filter_tasks_in_section(tasks, sections[cur_section_num].as_ref());
                if !tasks_in_cur_section.is_empty() {
                    found = Some(match pos {
                        AddPosition::Append => {
                            tasks_in_cur_section[tasks_in_cur_section.len() - 1].num + 1
                        }
                        AddPosition::Prepend => tasks_in_cur_section[0].num,
                    });
                    break;
                }

                cur_section_num -= 1;
            }

            found.unwrap_or(1)
        };

        new_lines = format!("\n{}", new_line);
        line_num = section.line_num_end + 1;
        new_content = text_add_line_in_str(&content, line_num, &new_lines);
    } else {
        let (new_lines_tmp, line_num_tmp, task_num_tmp) = match pos {
            AddPosition::Prepend => (
                format!("{}\n{}", new_line, section_tasks[0].line),
                section_tasks[0].line_num,
                section_tasks[0].num,
            ),
            AddPosition::Append => (
                format!(
                    "{}\n{}",
                    section_tasks[section_tasks.len() - 1].line,
                    new_line,
                ),
                section_tasks[section_tasks.len() - 1].line_num,
                section_tasks[section_tasks.len() - 1].num + 1,
            ),
        };
        new_lines = new_lines_tmp;
        line_num = line_num_tmp;
        task_num = task_num_tmp;
        new_content = text_replace_line_in_str(&content, line_num, &new_lines);
    }

    match content_setter.set_contents(new_content) {
        Err(e) => Err(e),
        Ok(()) => Ok((line_num, task_num)),
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
            Err(e) => panic!("{}", e),
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
            Err(e) => panic!("{}", e),
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
            Err(e) => panic!("{}", e),
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
            Err(e) => panic!("{}", e),
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
            Err(e) => panic!("{}", e),
        }
    }
}
