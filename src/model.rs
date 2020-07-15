#[derive(std::clone::Clone, Debug, PartialEq)]
pub struct Task {
    pub num: usize,
    pub name: String,
    pub is_completed: bool,
    pub line_num: usize,
    pub line: String,
    pub is_focused: bool,
}
