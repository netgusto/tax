use crate::cmd_list::cmd_list;
use crate::get_cmd_rank_arg;
use crate::services::{ContentGetter, ContentSetter, StringOutputer};
use crate::tasks::{get_all_tasks, replace_line_in_contents, toggle_line_focus};

pub fn cmd_focus(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    args: Vec<String>,
    focus: bool,
) -> Result<(), String> {
    let rank_one_based = match get_cmd_rank_arg(args)? {
        None => return cmd_list(outputer, content_getter),
        Some(rank) => rank,
    };

    let tasks = get_all_tasks(content_getter)?;
    if rank_one_based > tasks.len() {
        return Err(format!("Non existent task {}", rank_one_based));
    }

    let task = &tasks[rank_one_based - 1];

    if task.is_completed {
        outputer.info(format!(
            "Completed, cannot proceed: [{}] {}",
            task.num, task.name
        ));
        return Ok(());
    }

    if focus && task.is_focused {
        outputer.info(format!("Already focused: [{}] {}", task.num, task.name));
        return Ok(());
    } else if !focus && !task.is_focused {
        outputer.info(format!("Already blured: [{}] {}", task.num, task.name));
        return Ok(());
    }

    let replacement_line = toggle_line_focus(task.line.clone(), focus);
    let action = if focus { "Focused" } else { "Blurred" };
    outputer.info(format!("{}: [{}] {}", action, task.num, task.name));

    let replaced_content =
        replace_line_in_contents(content_getter, task.line_num, replacement_line)?;

    content_setter.set_contents(replaced_content)
}
