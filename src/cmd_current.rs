use crate::services::{ContentGetter, StringOutputer, TaskFormatter};
use crate::tasks::get_current_task;

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    task_formatter: &TaskFormatter,
    cycle: bool,
) -> Result<(), String> {
    if let Ok(Some((task, use_sections))) = get_current_task(content_getter, cycle) {
        outputer.info(&task_formatter.display_numbered_task(&task, use_sections, false))
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::test::{get_std_test_contents, ContentGetterMock, StringOutputerMock};

    #[test]
    fn test_cmd_current() {
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
                "[2] Standard unchecked focused\n"
            );
        }
    }
}
