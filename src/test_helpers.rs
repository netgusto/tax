#[cfg(test)]
pub mod test {
    use crate::model::Task;
    use crate::services::UserCmdRunner;
    use std::process::Command;

    pub fn env_getter_none(_: &str) -> Option<String> {
        None
    }

    #[allow(dead_code)]
    pub fn home_getter_guybrush() -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from("/home/guybrush"))
    }

    #[allow(dead_code)]
    pub fn env_getter_taxfile(name: &str) -> Option<String> {
        match name {
            "TAX_FILE" => Some("/path/to/overriden/taxfile".to_string()),
            _ => None,
        }
    }
    // ////////////////////////////////////////////////////////////////////////////
    // ContentGetterMock
    // ////////////////////////////////////////////////////////////////////////////
    pub struct ContentGetterMock {
        outcome: Result<String, String>,
    }

    impl ContentGetterMock {
        #[allow(dead_code)]
        pub fn new(outcome: Result<String, String>) -> Self {
            ContentGetterMock { outcome }
        }
    }

    impl crate::services::ContentGetter for ContentGetterMock {
        fn get_contents(&self) -> Result<String, String> {
            self.outcome.clone()
        }
    }

    // ////////////////////////////////////////////////////////////////////////////
    // ContentSetterMock
    // ////////////////////////////////////////////////////////////////////////////
    pub struct ContentSetterMock {
        pub content: Option<String>,
        outcome: Result<(), String>,
    }
    impl ContentSetterMock {
        pub fn new(outcome: Result<(), String>) -> Self {
            ContentSetterMock {
                content: None,
                outcome,
            }
        }
    }

    impl crate::services::ContentSetter for ContentSetterMock {
        fn set_contents(&mut self, contents: String) -> Result<(), String> {
            self.content = Some(contents);
            self.outcome.clone()
        }
    }

    // ////////////////////////////////////////////////////////////////////////////
    // StringOutputerMock
    // ////////////////////////////////////////////////////////////////////////////
    pub struct StringOutputerMock {
        info_buf: Vec<String>,
    }
    impl StringOutputerMock {
        #[allow(dead_code)]
        pub fn new() -> Self {
            StringOutputerMock { info_buf: vec![] }
        }
        #[allow(dead_code)]
        pub fn get_info(&self) -> String {
            self.info_buf.join("")
        }
    }
    impl crate::services::StringOutputer for StringOutputerMock {
        fn info(&mut self, s: &str) {
            self.info_buf.push(format!("{}\n", s));
        }
    }

    // ////////////////////////////////////////////////////////////////////////////
    // UserCmdRunnerMock
    // ////////////////////////////////////////////////////////////////////////////
    pub struct UserCmdRunnerMock {}

    impl UserCmdRunnerMock {
        pub fn new() -> Self {
            UserCmdRunnerMock {}
        }
    }

    impl UserCmdRunner for UserCmdRunnerMock {
        fn env_single_task<'a>(&self, _: &Task, cmd: &'a mut Command) -> &'a mut Command {
            cmd
        }

        fn build(&self, _: &str, _: &str, _: &str) -> Result<Option<Command>, String> {
            Ok(None)
        }

        fn run(&self, _: &mut Command) -> Result<(), String> {
            Ok(())
        }
    }

    // ////////////////////////////////////////////////////////////////////////
    // TaskFormatterMock
    // ////////////////////////////////////////////////////////////////////////
    #[allow(dead_code)]
    pub struct TaskFormatterMock {}

    #[allow(dead_code)]
    pub fn get_std_test_contents() -> (String, Vec<crate::model::Task>) {
        (
            vec![
                String::from("Not a task"),
                String::from("- [ ] Standard unchecked"),
                String::from("- [ ] **Standard unchecked focused**"),
                String::from("Also not a task"),
                String::from("- [x] Checked"),
                String::from("- [x] **Focused checked**"),
                String::from("- [ ] Standard unchecked // with comments"),
                String::from("- [ ] **Standard unchecked focused** // with comments"),
            ]
            .join("\n"),
            vec![
                crate::model::Task {
                    num: 1,
                    line_num: 2,
                    line: String::from("- [ ] Standard unchecked"),
                    name: String::from("Standard unchecked"),
                    plain_name: String::from("Standard unchecked"),
                    is_checked: false,
                    is_focused: false,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 2,
                    line_num: 3,
                    line: String::from("- [ ] **Standard unchecked focused**"),
                    name: String::from("**Standard unchecked focused**"),
                    plain_name: String::from("Standard unchecked focused"),
                    is_checked: false,
                    is_focused: true,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 3,
                    line_num: 5,
                    line: String::from("- [x] Checked"),
                    name: String::from("Checked"),
                    plain_name: String::from("Checked"),
                    is_checked: true,
                    is_focused: false,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 4,
                    line_num: 6,
                    line: String::from("- [x] **Focused checked**"),
                    name: String::from("**Focused checked**"),
                    plain_name: String::from("Focused checked"),
                    is_checked: true,
                    is_focused: true,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 5,
                    line_num: 7,
                    line: String::from("- [ ] Standard unchecked // with comments"),
                    name: String::from("Standard unchecked"),
                    plain_name: String::from("Standard unchecked"),
                    is_checked: false,
                    is_focused: false,
                    comment: Some(String::from("with comments")),
                    section: None,
                },
                crate::model::Task {
                    num: 6,
                    line_num: 8,
                    line: String::from("- [ ] **Standard unchecked focused** // with comments"),
                    name: String::from("**Standard unchecked focused**"),
                    plain_name: String::from("Standard unchecked focused"),
                    is_checked: false,
                    is_focused: true,
                    comment: Some(String::from("with comments")),
                    section: None,
                },
            ],
        )
    }
    #[allow(dead_code)]
    pub fn get_std_test_tasks() -> (String, Vec<crate::model::Task>) {
        (
            vec![
                String::from("- [ ] Standard unchecked"),
                String::from("- [ ] **Standard unchecked focused**"),
                String::from("- [x] Checked"),
                String::from("- [x] **Focused checked**"),
                String::from("- [ ] Standard unchecked // with comments"),
                String::from("- [ ] **Standard unchecked focused** // with comments"),
            ]
            .join("\n"),
            vec![
                crate::model::Task {
                    num: 1,
                    line_num: 1,
                    line: String::from("- [ ] Standard unchecked"),
                    name: String::from("Standard unchecked"),
                    plain_name: String::from("Standard unchecked"),
                    is_checked: false,
                    is_focused: false,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 2,
                    line_num: 2,
                    line: String::from("- [ ] **Standard unchecked focused**"),
                    name: String::from("**Standard unchecked focused**"),
                    plain_name: String::from("Standard unchecked focused"),
                    is_checked: false,
                    is_focused: true,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 3,
                    line_num: 3,
                    line: String::from("- [x] Checked"),
                    name: String::from("Checked"),
                    plain_name: String::from("Checked"),
                    is_checked: true,
                    is_focused: false,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 4,
                    line_num: 4,
                    line: String::from("- [x] **Focused checked**"),
                    name: String::from("**Focused checked**"),
                    plain_name: String::from("Focused checked"),
                    is_checked: true,
                    is_focused: true,
                    comment: None,
                    section: None,
                },
                crate::model::Task {
                    num: 5,
                    line_num: 5,
                    line: String::from("- [ ] Standard unchecked // with comments"),
                    name: String::from("Standard unchecked"),
                    plain_name: String::from("Standard unchecked"),
                    is_checked: false,
                    is_focused: false,
                    comment: Some(String::from("with comments")),
                    section: None,
                },
                crate::model::Task {
                    num: 6,
                    line_num: 6,
                    line: String::from("- [ ] **Standard unchecked focused** // with comments"),
                    name: String::from("**Standard unchecked focused**"),
                    plain_name: String::from("Standard unchecked focused"),
                    is_checked: false,
                    is_focused: true,
                    comment: Some(String::from("with comments")),
                    section: None,
                },
            ],
        )
    }
}
