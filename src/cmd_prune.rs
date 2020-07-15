use crate::services::{ContentGetter, ContentSetter, StringOutputer};
use crate::tasks::{format_numbered_task, get_closed_tasks, remove_lines_in_contents};

pub fn cmd_prune(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
) -> Result<(), String> {
    let tasks = get_closed_tasks(content_getter)?;

    if tasks.len() == 0 {
        outputer.info("No task to prune".to_string());
        return Ok(());
    }

    let line_nums: Vec<usize> = (&tasks).into_iter().map(|t| t.line_num).collect();
    let pruned_content = remove_lines_in_contents(content_getter, line_nums)?;

    content_setter.set_contents(pruned_content)?;

    outputer.info(format!(
        "Pruned {} task{}:",
        tasks.len(),
        if tasks.len() > 1 { "s" } else { "" }
    ));

    for task in &tasks {
        outputer.info(format_numbered_task(&task))
    }

    Ok(())
}
