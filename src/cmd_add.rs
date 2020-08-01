use crate::cmd_list;
use crate::model::Task;
use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{add_line_in_contents, get_all_tasks, get_comment, is_focused, remove_focus};

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd_add(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    task_parts: Vec<String>,
    pos: AddPosition,
) -> Result<(), String> {
    let task_name = task_parts.clone().join(" ");

    let tasks = get_all_tasks(content_getter)?;

    let (line_num, task_num, operation) = if tasks.len() == 0 {
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

    let new_line = format!("- [ ] {}", task_name);

    let result = add_line_in_contents(content_getter, line_num, new_line.clone())?;

    content_setter.set_contents(result)?;

    let (name_without_comment, comment) = get_comment(task_name.as_str());
    let is_task_focused = is_focused(name_without_comment.as_str());
    let plain_name = if is_task_focused {
        remove_focus(name_without_comment)
    } else {
        name_without_comment
    };

    let new_task = Task {
        name: task_name.clone(),
        plain_name: plain_name.clone(),
        comment: comment,
        line: new_line,
        line_num: line_num,
        is_checked: false,
        is_focused: is_task_focused,
        num: task_num,
    };

    match user_cmd_runner.build(
        String::from("add"),
        operation,
        format!("Added \"{}\"", task_name),
    ) {
        Ok(Some(mut cmd)) => {
            user_cmd_runner.run(user_cmd_runner.env_single_task(new_task, &mut cmd))?;
        }
        Ok(None) => (),
        Err(e) => return Err(e),
    };

    cmd_list(outputer, content_getter, task_formatter)
}
