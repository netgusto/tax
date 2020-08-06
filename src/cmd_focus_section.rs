use crate::services::{ContentGetter, ContentSetter, StringOutputer, UserCmdRunner};
use crate::tasks::{get_all_tasks, search_section, section_to_markdown, text_replace_line_in_str};

pub fn cmd(
    outputer: &mut dyn StringOutputer,
    content_getter: &dyn ContentGetter,
    content_setter: &mut dyn ContentSetter,
    _user_cmd_runner: &dyn UserCmdRunner,
    section_name: String,
    focus: bool,
) -> Result<(), String> {
    let (_, _, sections, _) = get_all_tasks(content_getter)?;

    let section = match search_section(section_name.as_ref(), &sections) {
        None => return Err(format!("Section not found: {}", section_name)),
        Some(section) => section,
    };

    if focus && section.is_focused {
        outputer.info(format!("Already focused: {}", section.plain_name));
        return Ok(());
    } else if !focus && !section.is_focused {
        outputer.info(format!("Already blured: {}", section.plain_name));
        return Ok(());
    }

    let mut updated_section = section.clone();
    updated_section.is_focused = focus;
    updated_section.line = section_to_markdown(&updated_section);

    outputer.info(format!(
        "{}: {}",
        if focus { "Focused" } else { "Blurred" },
        &section.plain_name
    ));

    let mut replaced_content = text_replace_line_in_str(
        &content_getter.get_contents()?,
        section.line_num,
        &updated_section.line,
    );

    // blur other focused sections

    for section_rc in sections {
        let section_cur = section_rc.as_ref();
        if section_cur.num == section.num {
            continue;
        }

        if section_cur.is_focused {
            let mut updated_section = section_cur.clone();
            updated_section.is_focused = false;
            let updated_line = section_to_markdown(&updated_section);

            replaced_content =
                text_replace_line_in_str(&replaced_content, updated_section.line_num, &updated_line)
        }
    }

    content_setter.set_contents(replaced_content)
}
