use crate::services::{ContentGetter, ContentSetter, StringOutputer, TaskFormatter, UserCmdRunner};
use crate::tasks::{get_closed_tasks, text_remove_lines_in_str};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    task_formatter: &TaskFormatter,
) -> Result<(), String> {
    let (tasks, use_sections, _, _) = get_closed_tasks(content_getter)?;

    if tasks.is_empty() {
        outputer.info("No task to prune");
        return Ok(());
    }

    let line_nums: Vec<usize> = (&tasks).iter().map(|t| t.line_num).collect();
    let pruned_content = text_remove_lines_in_str(&content_getter.get_contents()?, line_nums)?;

    content_setter.set_contents(pruned_content)?;

    let msg = format!(
        "Pruned {} task{}",
        tasks.len(),
        if tasks.len() > 1 { "s" } else { "" }
    );

    outputer.info(&msg);

    for task in &tasks {
        outputer.info(&task_formatter.display_numbered_task(task, use_sections, true))
    }

    match user_cmd_runner.build("prune", "PRUNE", &msg) {
        Ok(Some(mut cmd)) => {
            user_cmd_runner.run(&mut cmd)?;
        }
        Ok(None) => (),
        Err(e) => return Err(e),
    };

    Ok(())
}
