use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::get_open_tasks;

// use crate::model::Section;
// use std::rc::Rc;

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
) -> Result<(), String> {
    let (tasks, use_sections) = get_open_tasks(content_getter)?;
    // let mut current_section: Option<Rc<Section>> = None;
    let mut section_num = 0;

    for task in tasks {
        if use_sections {
            match &task.section {
                None => (),
                Some(rc) => {
                    let section = rc.as_ref();
                    if section_num != section.num {
                        outputer.info(format!(
                            "{}# {}",
                            if section_num == 0 { "" } else { "\n" },
                            section.name.clone()
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
            let content_getter_mock = &ContentGetterMock::new(Ok(Vec::new()));

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
