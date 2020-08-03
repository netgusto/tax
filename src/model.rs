use std::rc::Rc;

#[derive(std::clone::Clone, Debug, PartialEq)]
pub struct Task {
    pub num: usize,
    pub name: String,
    pub plain_name: String,
    pub comment: Option<String>,
    pub is_checked: bool,
    pub line_num: usize,
    pub line: String,
    pub is_focused: bool,
    pub section: Option<Rc<Section>>,
}

#[derive(std::clone::Clone, Debug, PartialEq)]
pub struct Section {
    pub num: usize,
    pub name: String,
    pub line_num: usize,
    pub line: String,
}
