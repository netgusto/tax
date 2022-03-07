use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::{filter_tasks_in_section, get_open_tasks};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
    all: bool,
) -> Result<(), String> {
    let (open_tasks, use_sections, _, focused_section) = get_open_tasks(content_getter)?;

    let mut other_tasks_hint: Option<String> = None;

    let filtered_tasks = if !all && use_sections && focused_section != None {
        let focused_section_unwrapped = focused_section.unwrap();
        let focused_section_ref = focused_section_unwrapped.as_ref();
        let ftasks = filter_tasks_in_section(&open_tasks, focused_section_ref);
        {
            let nb_diff = open_tasks.len() - ftasks.len();
            other_tasks_hint = if nb_diff > 0 {
                Some(format!(
                    "\n{} other open task{} outside of \"{}\".",
                    nb_diff,
                    if nb_diff > 1 { "s" } else { "" },
                    focused_section_ref.plain_name,
                ))
            } else {
                None
            };
            ftasks
        }
    } else {
        open_tasks
    };

    let mut section_num = 0;
    for task in filtered_tasks {
        if use_sections {
            // Display section header once
            match &task.section {
                None => (),
                Some(rc) => {
                    let section = rc.as_ref();
                    if section_num != section.num {
                        outputer.info(&format!(
                            "{}{} {}",
                            if section_num == 0 { "" } else { "\n" },
                            if section.is_focused {
                                task_formatter.display_bold_color_only("#")
                            } else {
                                "#".to_string()
                            },
                            if section.is_focused {
                                task_formatter.display_bold(&section.plain_name)
                            } else {
                                section.plain_name.clone()
                            }
                        ));

                        section_num = section.num;
                    }
                }
            }
        }

        outputer.info(&task_formatter.display_numbered_task(&task, false, true)); // false: disable inline section name, as header is displayed in list
    }

    match other_tasks_hint {
        None => (),
        Some(hint) => outputer.info(&hint),
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test::{get_std_test_contents, ContentGetterMock, StringOutputerMock};

    #[test]
    fn test_cmd_list() {
        let task_formatter = &TaskFormatter {
            supports_colors: false,
        };
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &ContentGetterMock::new(Ok("".to_string()));

            cmd(outputer_mock, content_getter_mock, task_formatter, false).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &ContentGetterMock::new(Ok(test_contents));

            cmd(outputer_mock, content_getter_mock, task_formatter, false).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[1] Standard unchecked\n[2] **Standard unchecked focused**\n[5] Standard unchecked\n[6] **Standard unchecked focused**\n"
            );
        }
    }
}
