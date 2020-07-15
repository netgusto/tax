#[macro_use]
extern crate lazy_static;
extern crate dirs;

use std::env;

mod services;
use services::ContentHandlerReal;
use services::StringOutputerReal;
use services::{env_getter_real, home_getter_real};
use services::{TaxfilePathGetter, TaxfilePathGetterReal};

mod model;
mod tasks;
mod test_helpers;

mod cmd_current;
use cmd_current::cmd_current;

mod cmd_edit;
use cmd_edit::cmd_edit;

mod cmd_list;
use cmd_list::cmd_list;

mod cmd_focus;
use cmd_focus::cmd_focus;

mod cmd_check;
use cmd_check::cmd_check;

mod cmd_prune;
use cmd_prune::cmd_prune;

mod cmd_cat;
use cmd_cat::cmd_cat;

mod cmd_which;
use cmd_which::cmd_which;

fn main() -> Result<(), String> {
    run_app(env::args().collect())
}

fn run_app(args: Vec<String>) -> Result<(), String> {
    let cmd: Option<&str> = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    let taxfile_path_getter_real = &TaxfilePathGetterReal {
        get_env: env_getter_real,
        get_home: home_getter_real,
    };

    let file_path = taxfile_path_getter_real.get_taxfile_path()?;
    let content_handler_real = &ContentHandlerReal { path: file_path };

    let outputer = &mut StringOutputerReal {};

    match cmd {
        Some("edit") => cmd_edit(taxfile_path_getter_real),

        Some("focus") => cmd_focus(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            true,
        ),
        Some("blur") => cmd_focus(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            false,
        ),

        Some("check") => cmd_check(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            true,
        ),
        Some("uncheck") => cmd_check(
            outputer,
            content_handler_real,
            content_handler_real,
            args,
            false,
        ),

        Some("list") => cmd_list(outputer, content_handler_real),
        Some("current") => cmd_current(outputer, content_handler_real, false),
        Some("cycle") => cmd_current(outputer, content_handler_real, true),

        Some("prune") => cmd_prune(outputer, content_handler_real, content_handler_real),

        Some("cat") => cmd_cat(outputer, content_handler_real),

        Some("which") => cmd_which(outputer, taxfile_path_getter_real),

        None => cmd_list(outputer, content_handler_real), // default: list
        _ => Err(format!("Unknown command \"{}\"", cmd.unwrap())),
    }
}

fn get_cmd_rank_arg(args: Vec<String>) -> Result<Option<usize>, String> {
    if args.len() == 2 {
        return Ok(None);
    }

    let rank_one_based = match str::parse::<usize>(args[2].as_str()) {
        Ok(0) | Err(_) => return Err(format!("Invalid task rank \"{}\"", args[2])),
        Ok(v) => v,
    };

    return Ok(Some(rank_one_based));
}
