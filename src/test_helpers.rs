#[cfg(test)]

pub fn env_getter_none(_: &str) -> Option<String> {
    return None;
}

#[allow(dead_code)]
pub fn home_getter_guybrush() -> Option<std::path::PathBuf> {
    return Some(std::path::PathBuf::from("/home/guybrush"));
}

#[allow(dead_code)]
pub fn env_getter_taxfile(name: &str) -> Option<String> {
    match name {
        "TAX_FILE" => Some("/path/to/overriden/taxfile".to_string()),
        _ => None,
    }
}
pub struct FileReaderMock {
    pub outcome: Result<Vec<String>, String>,
}
impl crate::services::ContentGetter for FileReaderMock {
    fn get_contents(&self) -> Result<Vec<String>, String> {
        self.outcome.clone()
    }
}
pub struct StringOutputerMock {
    pub info_buf: Vec<String>,
}
impl StringOutputerMock {
    #[allow(dead_code)]
    pub fn new() -> Self {
        return StringOutputerMock { info_buf: vec![] };
    }

    #[allow(dead_code)]
    pub fn get_info(&self) -> String {
        self.info_buf.join("")
    }
}
impl crate::services::StringOutputer for StringOutputerMock {
    fn info(&mut self, s: String) -> () {
        self.info_buf.push(String::from(format!("{}\n", s)));
    }
}

#[allow(dead_code)]
pub fn get_std_test_contents(tasks_only: bool) -> (Vec<String>, Vec<crate::model::Task>) {
    (
        if tasks_only {
            vec![
                String::from("- [ ] Standard unchecked"),
                String::from("- [ ] **Standard unchecked focused**"),
                String::from("- [x] Checked"),
                String::from("- [x] **Focused checked**"),
                String::from("- [ ] Standard unchecked // with comments"),
                String::from("- [ ] **Standard unchecked focused** // with comments"),
            ]
        } else {
            vec![
                String::from("# Not a task"),
                String::from("- [ ] Standard unchecked"),
                String::from("- [ ] **Standard unchecked focused**"),
                String::from("Also not a task"),
                String::from("- [x] Checked"),
                String::from("- [x] **Focused checked**"),
                String::from("- [ ] Standard unchecked // with comments"),
                String::from("- [ ] **Standard unchecked focused** // with comments"),
            ]
        },
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
            },
            crate::model::Task {
                num: 3,
                line_num: 4,
                line: String::from("- [ ] **Standard unchecked focused**"),
                name: String::from("**Standard unchecked focused**"),
                plain_name: String::from("Standard unchecked focused"),
                is_checked: false,
                is_focused: true,
                comment: None,
            },
            crate::model::Task {
                num: 5,
                line_num: 7,
                line: String::from("- [x] Checked"),
                name: String::from("Checked"),
                plain_name: String::from("Checked"),
                is_checked: true,
                is_focused: false,
                comment: None,
            },
            crate::model::Task {
                num: 6,
                line_num: 8,
                line: String::from("- [x] **Focused checked**"),
                name: String::from("**Focused checked**"),
                plain_name: String::from("Focused checked"),
                is_checked: true,
                is_focused: true,
                comment: None,
            },
            crate::model::Task {
                num: 7,
                line_num: 9,
                line: String::from("- [ ] Standard unchecked // with comments"),
                name: String::from("Standard unchecked"),
                plain_name: String::from("Standard unchecked"),
                is_checked: false,
                is_focused: false,
                comment: Some(String::from("with comments")),
            },
            crate::model::Task {
                num: 8,
                line_num: 10,
                line: String::from("- [ ] **Standard unchecked focused** // with comments"),
                name: String::from("**Standard unchecked focused**"),
                plain_name: String::from("Standard unchecked focused"),
                is_checked: false,
                is_focused: true,
                comment: Some(String::from("with comments")),
            },
        ],
    )
}
