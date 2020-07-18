use crate::model::Task;

use std::env;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait StringOutputer {
    fn info(&mut self, s: String) -> ();
}

pub struct StringOutputerReal {}
impl StringOutputer for StringOutputerReal {
    fn info(&mut self, s: String) -> () {
        println!("{}", s);
    }
}

pub type EnvGetter = fn(&str) -> Option<String>;
pub fn env_getter_real(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

pub type HomeGetter = fn() -> Option<PathBuf>;
pub fn home_getter_real() -> Option<PathBuf> {
    match dirs::home_dir() {
        None => return None,
        Some(h) => Some(h),
    }
}

pub trait TaxfilePathGetter {
    fn get_taxfile_path(&self) -> Result<String, String>;
    fn get_taxfile_dir(&self) -> Result<String, String>;
}

pub struct TaxfilePathGetterReal {
    pub get_env: EnvGetter,
    pub get_home: HomeGetter,
}

impl TaxfilePathGetter for TaxfilePathGetterReal {
    fn get_taxfile_path(&self) -> Result<String, String> {
        match get_env_var_if_not_empty("TAX_FILE", self.get_env) {
            Some(v) => Ok(v),
            None => match (self.get_home)() {
                None => Err(String::from("Could not find home dir")),
                Some(home) => Ok(String::from(
                    home.join(Path::new("tasks.md")).to_str().unwrap(),
                )),
            },
        }
    }

    fn get_taxfile_dir(&self) -> Result<String, String> {
        let taxfile = self.get_taxfile_path()?;
        let mut taxfile_pathbuf = PathBuf::from(taxfile);
        taxfile_pathbuf.pop();
        Ok(String::from(taxfile_pathbuf.to_str().unwrap()))
    }
}

pub struct ContentHandlerReal {
    pub path: String,
}

pub trait ContentGetter {
    fn get_contents(&self) -> Result<Vec<String>, String>;
}

impl ContentGetter for ContentHandlerReal {
    fn get_contents(&self) -> Result<Vec<String>, String> {
        match File::open(&self.path) {
            Err(_) => Err(format!("Could not open file {}", &self.path)),
            Ok(f) => {
                let reader = BufReader::new(f);
                let lines_result = reader.lines().collect::<Result<_, _>>();
                match lines_result {
                    Err(_) => Err(format!("Could not read file {}", &self.path)),
                    Ok(lines) => Ok(lines),
                }
            }
        }
    }
}

pub trait ContentSetter {
    fn set_contents(&self, contents: String) -> Result<(), String>;
}

impl ContentSetter for ContentHandlerReal {
    fn set_contents(&self, contents: String) -> Result<(), String> {
        match fs::write(&self.path, contents) {
            Ok(_) => Ok(()),
            Err(_) => Err(String::from("Unable to write file")),
        }
    }
}

pub struct UserCmdRunnerReal {
    pub taxfile_path_getter: &'static dyn TaxfilePathGetter,
    pub get_env: EnvGetter,
}

pub trait UserCmdRunner {
    fn env_single_task<'a>(&self, task: Task, cmd: &'a mut Command) -> &'a mut Command;
    fn build(
        &self,
        cmd: String,
        operation: String,
        message: String,
    ) -> Result<Option<Command>, String>;
    fn run(&self, cmd: &mut Command) -> Result<(), String>;
}

impl UserCmdRunner for UserCmdRunnerReal {
    fn env_single_task<'a>(&self, task: Task, cmd: &'a mut Command) -> &'a mut Command {
        cmd.env("TAX_TASK_NUM", format!("{}", task.num))
            .env("TAX_TASK_NAME", &task.name)
            .env("TAX_TASK_LINE", &task.line)
            .env("TAX_TASK_LINE_NUM", format!("{}", task.line_num))
            .env("TAX_TASK_CHECKED", if task.is_checked { "1" } else { "0" })
            .env("TAX_TASK_FOCUSED", if task.is_focused { "1" } else { "0" })
    }

    fn build(
        &self,
        cmd: String,
        operation: String,
        message: String,
    ) -> Result<Option<Command>, String> {
        let sh_path = match which::which("sh") {
            Ok(path) => path,
            Err(_) => return Err(String::from("Could not find sh")),
        };

        match get_env_var_if_not_empty("TAX_CHANGE_CMD", self.get_env) {
            Some(change_cmd) => {
                let mut cmd_obj = Command::new(sh_path);
                cmd_obj
                    .arg("-c")
                    .arg(change_cmd)
                    .env("TAX_FILE", self.taxfile_path_getter.get_taxfile_path()?)
                    .env(
                        "TAX_FILE_FOLDER",
                        self.taxfile_path_getter.get_taxfile_dir()?,
                    )
                    .env("TAX_CMD", cmd)
                    .env("TAX_OPERATION", operation)
                    .env("TAX_MESSAGE", message);
                return Ok(Some(cmd_obj));
            }
            None => Ok(None),
        }
    }

    fn run(&self, cmd: &mut Command) -> Result<(), String> {
        match cmd.spawn() {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e)),
            },
            Err(e) => Err(format!("{}", e)),
        }
    }
}

fn get_env_var_if_not_empty(name: &str, get_env: EnvGetter) -> Option<String> {
    match (get_env)(name) {
        Some(v) => {
            if v.trim().len() == 0 {
                None
            } else {
                Some(v)
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helpers::{env_getter_none, env_getter_taxfile, home_getter_guybrush};

    #[test]
    fn test_taxfile_path_getter_real() {
        let path_getter_noenv = &TaxfilePathGetterReal {
            get_env: env_getter_none,
            get_home: home_getter_guybrush,
        };
        assert_eq!(
            path_getter_noenv.get_taxfile_path(),
            Ok(String::from("/home/guybrush/tasks.md"))
        );

        let path_getter_yesenv = &TaxfilePathGetterReal {
            get_env: env_getter_taxfile,
            get_home: home_getter_guybrush,
        };

        assert_eq!(
            path_getter_yesenv.get_taxfile_path(),
            Ok(String::from("/path/to/overriden/taxfile"))
        );
    }
}
