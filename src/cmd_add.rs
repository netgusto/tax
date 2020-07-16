use crate::services::{ContentGetter, ContentSetter, StringOutputer};
use crate::tasks::{add_line_in_contents, get_all_tasks};

pub enum AddPosition {
    Append,
    Prepend,
}

pub fn cmd_add(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &dyn ContentSetter,
    args: Vec<String>,
    pos: AddPosition,
) -> Result<(), String> {
    if args.len() < 3 {
        return Err(String::from("Please specify a task"));
    }

    let task_parts: Vec<String> = args.iter().skip(2).map(|s| (*s).clone()).collect();
    let task = task_parts.clone().join(" ");

    let tasks = get_all_tasks(content_getter)?;

    let line_num = if tasks.len() == 0 {
        1
    } else {
        match pos {
            AddPosition::Prepend => tasks[0].line_num,
            AddPosition::Append => tasks[tasks.len() - 1].line_num + 1,
        }
    };

    let result = add_line_in_contents(content_getter, line_num, format!("- [ ] {}", task))?;
    outputer.info(format!("{}", result));

    content_setter.set_contents(result)
}
