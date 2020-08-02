use crate::cmd_list;
use crate::model::Task;
use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{
    get_all_tasks, task_to_markdown, text_add_line_in_contents, text_get_comment, text_is_focused,
    text_remove_focus,
};

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    task_parts: Vec<String>,
    pos: AddPosition,
) -> Result<(), String> {
    let task_name = task_parts.join(" ");

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
        comment: comment,
        line: String::from(""),
        line_num: line_num,
        is_checked: false,
        is_focused: is_task_focused,
        num: task_num,
    };

    new_task.line = task_to_markdown(&new_task);
    let result = text_add_line_in_contents(content_getter, line_num, new_task.line.clone())?;
    content_setter.set_contents(result)?;

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

    cmd_list::cmd(outputer, content_getter, task_formatter)
}
