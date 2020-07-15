use crate::services::{ContentGetter, StringOutputer};
use crate::tasks::get_current_task;

pub fn cmd_current(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    cycle: bool,
) -> Result<(), String> {
    match get_current_task(content_getter, cycle) {
        Ok(Some(task)) => outputer.info(format!("[{}] {}", task.num, task.name)),
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
        // Empty contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(Vec::new()),
            };

            cmd_current(outputer_mock, content_getter_mock, false).unwrap();
            assert_eq!(outputer_mock.get_info(), "");
        }

        // Std contents
        {
            let outputer_mock = &mut StringOutputerMock::new();
            let (test_contents, _) = get_std_test_contents();
            let content_getter_mock = &FileReaderMock {
                outcome: Ok(test_contents),
            };

            cmd_current(outputer_mock, content_getter_mock, false).unwrap();
            assert_eq!(
                outputer_mock.get_info(),
                "[3] **Standard unchecked focused**\n"
            );
        }
    }
}
