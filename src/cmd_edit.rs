use crate::services::TaxfilePathGetter;
use std::env;
use std::process::Command;

pub fn cmd_edit(taxfile_path_getter: &dyn TaxfilePathGetter) -> Result<(), String> {
    let str_file_path = taxfile_path_getter.get_taxfile_path().unwrap();

    let res = env::var("EDITOR");
    if !res.is_ok() {
        return Err(String::from(
            "Please set $EDITOR in environment to use \"edit\".",
        ));
    }

    let editor = res.unwrap();

    if let Ok(mut child) = Command::new(editor).arg(str_file_path).spawn() {
        match child.wait() {
            Ok(_) => (),
            Err(_) => {
                return Err(String::from("Could not run $EDITOR"));
            }
        }
    } else {
        return Err(String::from("Could not run $EDITOR"));
    }

    Ok(())
}
