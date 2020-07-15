use crate::services::{ContentGetter, StringOutputer};

pub fn cmd_cat(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
) -> Result<(), String> {
    let content = content_getter.get_contents()?;
    for line in &content {
        outputer.info(line.to_string());
    }

    Ok(())
}
