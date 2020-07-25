use crate::cmd_list::cmd_list;
use crate::get_cmd_rank_arg;
use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{get_all_tasks, remove_focus, replace_line_in_contents, toggle_line_focus};

pub fn cmd_focus(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    args: Vec<String>,
    focus: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(outputer, content_getter, task_formatter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_checked {
        outputer.info(format!(
            "Task is completed, cannot proceed: {}",
            task_formatter.display_numbered_task(&task)
        ));
        return Ok(());
    }

    if focus && task.is_focused {
        outputer.info(format!(
            "Already focused: {}",
            task_formatter.display_numbered_task(&task)
        ));
        return Ok(());
    } else if !focus && !task.is_focused {
        outputer.info(format!(
            "Already blured: {}",
            task_formatter.display_numbered_task(&task)
        ));
        return Ok(());
    }

    let replacement_line = toggle_line_focus(task.line.clone(), focus);

    let replaced_content =
        replace_line_in_contents(content_getter, task.line_num, replacement_line.clone())?;

    let result = content_setter.set_contents(replaced_content);

    let mut updated_task = task.clone();
    updated_task.line = replacement_line;
    updated_task.is_focused = focus;
    updated_task.name = remove_focus(task.name.clone());

    let action = if focus { "Focused" } else { "Blurred" };
    outputer.info(format!(
        "{}: {}",
        action,
        task_formatter.display_numbered_task(&updated_task)
    ));

    match user_cmd_runner.build(
        String::from("focus"),
        String::from(if focus { "FOCUS" } else { "BLUR" }),
        format!(
            "{} \"{}\"",
            if focus { "Focused" } else { "Blurred" },
            task.name
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
