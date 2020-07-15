use std::env;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

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
}

pub struct TaxfilePathGetterReal {
    pub get_env: EnvGetter,
    pub get_home: HomeGetter,
}

impl TaxfilePathGetter for TaxfilePathGetterReal {
    fn get_taxfile_path(&self) -> Result<String, String> {
        match (self.get_env)("TAXFILE") {
            Some(v) => Ok(v),
            None => match (self.get_home)() {
                None => Err(String::from("Could not find home dir")),
                Some(home) => Ok(String::from(
                    home.join(Path::new("taxfile")).to_str().unwrap(),
                )),
            },
        }
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
            Ok(String::from("/home/guybrush/taxfile"))
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
