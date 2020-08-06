use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::{filter_tasks_in_section, get_open_tasks};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
) -> Result<(), String> {
    let (tasks, use_sections, _, focused_section) = get_open_tasks(content_getter)?;

    let filtered_tasks = if use_sections && focused_section != None {
        filter_tasks_in_section(&tasks, focused_section.unwrap().as_ref())
    } else {
        tasks
    };

    let mut section_num = 0;
    for task in filtered_tasks {
        if use_sections {
            match &task.section {
                None => (),
                Some(rc) => {
                    let section = rc.as_ref();
                    if section_num != section.num {
                        outputer.info(format!(
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
        outputer.info(task_formatter.display_numbered_task(&task, false))
    }

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

            cmd(outputer_mock, content_getter_mock, task_formatter).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &ContentGetterMock::new(Ok(test_contents));

            cmd(outputer_mock, content_getter_mock, task_formatter).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[1] Standard unchecked\n[2] **Standard unchecked focused**\n[5] Standard unchecked\n[6] **Standard unchecked focused**\n"
            );
        }
    }
}
