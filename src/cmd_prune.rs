use crate::services::{ContentGetter, ContentSetter, StringOutputer, UserCmdRunner};
use crate::tasks::{format_numbered_task, get_closed_tasks, remove_lines_in_contents};

pub fn cmd_prune(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
) -> Result<(), String> {
    let tasks = get_closed_tasks(content_getter)?;

    if tasks.len() == 0 {
        outputer.info("No task to prune".to_string());
        return Ok(());
    }

    let line_nums: Vec<usize> = (&tasks).into_iter().map(|t| t.line_num).collect();
    let pruned_content = remove_lines_in_contents(content_getter, line_nums)?;

    content_setter.set_contents(pruned_content)?;

    let msg = format!(
        "Pruned {} task{}",
        tasks.len(),
        if tasks.len() > 1 { "s" } else { "" }
    );

    outputer.info(msg.clone());

    for task in &tasks {
        outputer.info(format_numbered_task(&task))
    }

    match user_cmd_runner.build(String::from("prune"), String::from("PRUNE"), msg) {
        Ok(Some(mut cmd)) => {
            user_cmd_runner.run(&mut cmd)?;
        }
        Ok(None) => (),
        Err(e) => return Err(e),
    };

    Ok(())
}
