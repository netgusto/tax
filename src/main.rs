#[macro_use]
extern crate lazy_static;
extern crate dirs;

use std::env;

mod services;
use services::ContentHandlerReal;
use services::StringOutputerReal;
use services::{env_getter_real, home_getter_real};
use services::{TaxfilePathGetter, TaxfilePathGetterReal, UserCmdRunnerReal};

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

mod cmd_add;
use cmd_add::cmd_add;

fn main() -> Result<(), String> {
    run_app(env::args().collect())
}

fn run_app(args: Vec<String>) -> Result<(), String> {
    let cmd: Option<&str> = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        None
    };

    let taxfile_path_getter = &TaxfilePathGetterReal {
        get_env: env_getter_real,
        get_home: home_getter_real,
    };

    let user_cmd_runner = &UserCmdRunnerReal {
        taxfile_path_getter: taxfile_path_getter,
    };

    let file_path = taxfile_path_getter.get_taxfile_path()?;
    let content_handler = &ContentHandlerReal { path: file_path };

    let outputer = &mut StringOutputerReal {};

    match cmd {
        Some("edit") => cmd_edit(taxfile_path_getter, user_cmd_runner),

        Some("focus") => cmd_focus(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            true,
        ),
        Some("blur") | Some("unfocus") => cmd_focus(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            false,
        ),

        Some("check") => cmd_check(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            true,
        ),
        Some("uncheck") => cmd_check(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            false,
        ),

        Some("list") => cmd_list(outputer, content_handler),
        Some("current") => cmd_current(outputer, content_handler, false),
        Some("cycle") => cmd_current(outputer, content_handler, true),

        Some("prune") | Some("purge") => {
            cmd_prune(outputer, content_handler, content_handler, user_cmd_runner)
        }

        Some("cat") | Some("view") => cmd_cat(outputer, content_handler),

        Some("which") => cmd_which(outputer, taxfile_path_getter),

        Some("add") | Some("push") | Some("prepend") => cmd_add(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            cmd_add::AddPosition::Prepend,
        ),

        Some("append") => cmd_add(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            args,
            cmd_add::AddPosition::Append,
        ),

        None => cmd_list(outputer, content_handler), // default: list
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
