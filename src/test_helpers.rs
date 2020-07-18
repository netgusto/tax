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
pub fn get_std_test_contents() -> (Vec<String>, Vec<crate::model::Task>) {
    (
        vec![
            String::from("# Not a task"),
            String::from("- [ ] Standard unchecked"),
            String::from("- [] Collapsed unchecked"),
            String::from("- [ ] **Standard unchecked focused**"),
            String::from("* [ ] Star unchecked"),
            String::from("Also not a task"),
            String::from("- [x] Checked"),
            String::from("- [x] **Focused checked**"),
        ],
        vec![
            crate::model::Task {
                num: 1,
                line_num: 2,
                line: String::from("- [ ] Standard unchecked"),
                name: String::from("Standard unchecked"),
                is_checked: false,
                is_focused: false,
            },
            crate::model::Task {
                num: 2,
                line_num: 3,
                line: String::from("- [] Collapsed unchecked"),
                name: String::from("Collapsed unchecked"),
                is_checked: false,
                is_focused: false,
            },
            crate::model::Task {
                num: 3,
                line_num: 4,
                line: String::from("- [ ] **Standard unchecked focused**"),
                name: String::from("**Standard unchecked focused**"),
                is_checked: false,
                is_focused: true,
            },
            crate::model::Task {
                num: 4,
                line_num: 5,
                line: String::from("* [ ] Star unchecked"),
                name: String::from("Star unchecked"),
                is_checked: false,
                is_focused: false,
            },
            crate::model::Task {
                num: 5,
                line_num: 7,
                line: String::from("- [x] Checked"),
                name: String::from("Checked"),
                is_checked: true,
                is_focused: false,
            },
            crate::model::Task {
                num: 6,
                line_num: 8,
                line: String::from("- [x] **Focused checked**"),
                name: String::from("**Focused checked**"),
                is_checked: true,
                is_focused: true,
            },
        ],
    )
}
