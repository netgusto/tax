use crate::services::{ContentGetter, StringOutputer};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
) -> Result<(), String> {
    let content = content_getter.get_contents()?;
    for line in content.lines() {
        outputer.info(line.to_string());
    }

    Ok(())
}
