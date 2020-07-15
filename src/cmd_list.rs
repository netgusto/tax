use crate::services::{ContentGetter, StringOutputer};
use crate::tasks::{format_numbered_task, get_open_tasks};

pub fn cmd_list(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
) -> Result<(), String> {
    let tasks = get_open_tasks(content_getter)?;
    for task in tasks {
        outputer.info(format_numbered_task(&task))
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{get_std_test_contents, FileReaderMock, StringOutputerMock};

    #[test]
    fn test_cmd_list() {
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd_list(outputer_mock, content_getter_mock).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd_list(outputer_mock, content_getter_mock).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[1] Standard unchecked\n[2] Collapsed unchecked\n[3] **Standard unchecked focused**\n[4] Star unchecked\n"
            );
        }
    }
}
