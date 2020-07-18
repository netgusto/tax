use crate::model::Task;
use crate::services::{ContentGetter, ContentSetter, StringOutputer, UserCmdRunner};
use crate::tasks::{add_line_in_contents, get_all_tasks};

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd_add(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    user_cmd_runner: &dyn UserCmdRunner,
    args: Vec<String>,
    pos: AddPosition,
) -> Result<(), String> {
    if args.len() < 3 {
        return Err(String::from("Please specify a task"));
    }

    let task_parts: Vec<String> = args.iter().skip(2).map(|s| (*s).clone()).collect();
    let task_name = task_parts.clone().join(" ");

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

    let new_line = format!("- [ ] {}", task_name);

    let result = add_line_in_contents(content_getter, line_num, new_line.clone())?;
    outputer.info(format!("{}", result));

    let set_result = content_setter.set_contents(result);

    let new_task = Task {
        name: task_name.clone(),
        line: new_line,
        line_num: line_num,
        is_completed: false,
        is_focused: false,
        num: task_num,
    };

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

    set_result
}
