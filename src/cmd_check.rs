use super::get_cmd_rank_arg;
use crate::cmd_list::cmd_list;
use crate::services::{ContentGetter, ContentSetter, StringOutputer, UserCmdRunner};
use crate::tasks::{get_all_tasks, replace_line_in_contents, toggle_line_completion};

pub fn cmd_check(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    args: Vec<String>,
    completed: bool,
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

    if completed && task.is_completed {
        outputer.info(format!("Already checked: [{}] {}", task.num, task.name));
        return Ok(());
    } else if !completed && !task.is_completed {
        outputer.info(format!("Already unckecked: [{}] {}", task.num, task.name));
        return Ok(());
    }

    let checked_line = toggle_line_completion(task.line.clone(), completed);
    let action = if completed { "Checked" } else { "Unchecked" };
    outputer.info(format!("{}: [{}] {}", action, task.num, task.name));

    let replaced_content =
        replace_line_in_contents(content_getter, task.line_num, checked_line.clone())?;

    let result = content_setter.set_contents(replaced_content);

    let mut updated_task = task.clone();
    updated_task.is_completed = completed;
    updated_task.line = checked_line;

    match user_cmd_runner.build(
        String::from("check"),
        String::from(if completed { "CHECK" } else { "UNCHECK" }),
        format!(
            "Marked \"{}\" as {}",
            task.name,
            if completed { "done" } else { "not done" }
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
