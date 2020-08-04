use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{get_all_tasks, task_to_markdown, text_add_focus, text_replace_line_in_str};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    rank_one_based: usize,
    focus: bool,
) -> Result<(), String> {
    let (tasks, use_sections, _) = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_checked {
        outputer.info(format!(
            "Task is completed, cannot proceed: {}",
            task_formatter.display_numbered_task(&task, use_sections)
        ));
        return Ok(());
    }

    if focus && task.is_focused {
        outputer.info(format!(
            "Already focused: {}",
            task_formatter.display_numbered_task(&task, use_sections)
        ));
        return Ok(());
    } else if !focus && !task.is_focused {
        outputer.info(format!(
            "Already blured: {}",
            task_formatter.display_numbered_task(&task, use_sections)
        ));
        return Ok(());
    }

    let mut updated_task = task.clone();
    updated_task.is_focused = focus;
    updated_task.name = if focus {
        text_add_focus(task.plain_name.as_str())
    } else {
        task.plain_name.clone()
    };

    updated_task.line = task_to_markdown(&updated_task);

    let replaced_content = text_replace_line_in_str(
        &content_getter.get_contents()?,
        task.line_num,
        &updated_task.line,
    );

    let result = content_setter.set_contents(replaced_content);

    let action = if focus { "Focused" } else { "Blurred" };
    outputer.info(format!(
        "{}: {}",
        action,
        task_formatter.display_numbered_task(&updated_task, use_sections)
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
