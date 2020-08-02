use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{get_all_tasks, replace_line_in_contents, toggle_line_completion};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    rank_one_based: usize,
    checked: bool,
) -> Result<(), String> {
    let tasks = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if checked && task.is_checked {
        outputer.info(format!(
            "Already checked: {}",
            task_formatter.display_numbered_task(&task)
        ));
        return Ok(());
    } else if !checked && !task.is_checked {
        outputer.info(format!(
            "Already unckecked: {}",
            task_formatter.display_numbered_task(&task)
        ));
        return Ok(());
    }

    let checked_line = toggle_line_completion(task.line.clone(), checked);
    let action = if checked { "Checked" } else { "Unchecked" };
    outputer.info(format!(
        "{}: {}",
        action,
        task_formatter.display_numbered_task(&task)
    ));

    let replaced_content =
        replace_line_in_contents(content_getter, task.line_num, checked_line.clone())?;

    let result = content_setter.set_contents(replaced_content);

    let mut updated_task = task.clone();
    updated_task.is_checked = checked;
    updated_task.line = checked_line;

    match user_cmd_runner.build(
        String::from("check"),
        String::from(if checked { "CHECK" } else { "UNCHECK" }),
        format!(
            "Marked \"{}\" as {}",
            task.name,
            if checked { "done" } else { "not done" }
        ),
    ) {
        Ok(Some(mut cmd)) => {
            user_cmd_runner.run(user_cmd_runner.env_single_task(updated_task, &mut cmd))?;
        }
        Ok(None) => (),
        Err(e) => return Err(e),
    };

    result
}
