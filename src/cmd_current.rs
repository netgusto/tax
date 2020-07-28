use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::get_current_task;

pub fn cmd_current(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
    cycle: bool,
) -> Result<(), String> {
    match get_current_task(content_getter, cycle) {
        Ok(Some(task)) => outputer.info(task_formatter.display_numbered_task(&task)),
        _ => (),
    };
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::{get_std_test_contents, FileReaderMock, StringOutputerMock};

    #[test]
    fn test_cmd_current() {
        let task_formatter = &TaskFormatter {
            supports_colors: false,
        };
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd_current(outputer_mock, content_getter_mock, &task_formatter, false).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents(false);
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd_current(outputer_mock, content_getter_mock, &task_formatter, false).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[3] **Standard unchecked focused**\n"
            );
        }
    }
}
