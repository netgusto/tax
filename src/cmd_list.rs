use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::get_open_tasks;

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
) -> Result<(), String> {
    let tasks = get_open_tasks(content_getter)?;
    for task in tasks {
        outputer.info(task_formatter.display_numbered_task(&task))
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{get_std_test_contents, FileReaderMock, StringOutputerMock};

    #[test]
    fn test_cmd_list() {
        let task_formatter = &TaskFormatter {
            supports_colors: false,
        };
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd(outputer_mock, content_getter_mock, task_formatter).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd(outputer_mock, content_getter_mock, task_formatter).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[1] Standard unchecked\n[2] **Standard unchecked focused**\n[5] Standard unchecked\n[6] **Standard unchecked focused**\n"
            );
        }
    }
}
