use crate::services::{TaxfilePathGetter, UserCmdRunner};
use std::env;
use std::process::Command;

pub fn cmd(
    taxfile_path_getter: &dyn TaxfilePathGetter,
    user_cmd_runner: &dyn UserCmdRunner,
) -> Result<(), String> {
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
            Ok(_) => {
                match user_cmd_runner.build("edit", "EDIT", "Manually edited tasks") {
                    Ok(Some(mut cmd)) => {
                        user_cmd_runner.run(&mut cmd)?;
                    }
                    Ok(None) => (),
                    Err(e) => return Err(e),
                };
            }
            Err(_) => {
                return Err(String::from("Could not run $EDITOR"));
            }
        }
    } else {
        return Err(String::from("Could not run $EDITOR"));
    }

    Ok(())
}
