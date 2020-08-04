use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{get_all_tasks, task_to_markdown, text_replace_line_in_str};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
    rank_one_based: usize,
    checked: bool,
) -> Result<(), String> {
    let (tasks, use_sections, _) = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if checked && task.is_checked {
        outputer.info(format!(
            "Already checked: {}",
            task_formatter.display_numbered_task(&task, use_sections)
        ));
        return Ok(());
    } else if !checked && !task.is_checked {
        outputer.info(format!(
            "Already unckecked: {}",
            task_formatter.display_numbered_task(&task, use_sections)
        ));
        return Ok(());
    }

    let mut updated_task = task.clone();
    updated_task.is_checked = checked;
    updated_task.line = task_to_markdown(&updated_task);

    let action = if checked { "Checked" } else { "Unchecked" };
    outputer.info(format!(
        "{}: {}",
        action,
        task_formatter.display_numbered_task(&updated_task, use_sections)
    ));

    let replaced_content = text_replace_line_in_str(
        &content_getter.get_contents()?,
        updated_task.line_num,
        &updated_task.line,
    );

    let result = content_setter.set_contents(replaced_content);

    match user_cmd_runner.build(
        String::from("check"),
        String::from(if checked { "CHECK" } else { "UNCHECK" }),
        format!(
            "Marked \"{}\" as {}",
            updated_task.name,
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
