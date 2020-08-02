#[macro_use]
extern crate lazy_static;

use clap::{crate_version, value_t, App, Arg, ArgMatches};
use colored::control::SHOULD_COLORIZE;

mod services;
use services::{
    env_getter_real, home_getter_real, ContentHandlerReal, StringOutputerReal, TaskFormatter,
    TaxfilePathGetter, TaxfilePathGetterReal, UserCmdRunnerReal,
};

mod model;
mod tasks;
mod test_helpers;

mod cmd_add;
mod cmd_cat;
mod cmd_check;
mod cmd_current;
mod cmd_edit;
mod cmd_focus;
mod cmd_list;
mod cmd_prune;
mod cmd_which;

fn main() -> Result<(), String> {
    run_app(get_arg_matches())
}

fn get_arg_matches() -> ArgMatches<'static> {
    App::new("Tax")
        .version(crate_version!())
        .about("CLI Task List Manager")
        .subcommand(App::new("edit").about("Edit the current task list in $EDITOR"))
        .subcommand(
            App::new("focus").about("Focus the given task").arg(
                Arg::with_name("task-index")
                    .index(1)
                    .required(true)
                    .help("Number of the task to focus"),
            ),
        )
        .subcommand(
            App::new("blur")
                .alias("unfocus")
                .about("Blur the given task")
                .arg(
                    Arg::with_name("task-index")
                        .index(1)
                        .required(true)
                        .help("Number of the task to blur"),
                ),
        )
        .subcommand(
            App::new("check")
                .about("Mark the given task as completed")
                .arg(
                    Arg::with_name("task-index")
                        .index(1)
                        .required(true)
                        .help("Number of the task to mark as completed"),
                ),
        )
        .subcommand(
            App::new("uncheck")
                .about("Mark the given task as not completed")
                .arg(
                    Arg::with_name("task-index")
                        .index(1)
                        .required(true)
                        .help("Number of the task to mark as not completed"),
                ),
        )
        .subcommand(
            App::new("list")
                .alias("ls")
                .about("Print all the tasks of the list, completed or not"),
        )
        .subcommand(
            App::new("current").about("Print the first open (focused if any) task of the list"),
        )
        .subcommand(
            App::new("cycle")
                .about("Like current, but changes task every minute if no task is focused"),
        )
        .subcommand(
            App::new("prune")
                .alias("purge")
                .about("Remove all completed tasks from the task list"),
        )
        .subcommand(App::new("cat").alias("view").about("Print the task file"))
        .subcommand(App::new("which").about("Print the path of the current task list file"))
        .subcommand(
            App::new("add")
                .alias("push")
                .alias("prepend")
                .about("Add the given task at the top of the task list")
                .arg(
                    Arg::with_name("task-name")
                        .index(1)
                        .required(true)
                        .multiple(true)
                        .help("Name of the task to add"),
                ),
        )
        .subcommand(
            App::new("append")
                .about("Add the given task at the bottom of the task list")
                .arg(
                    Arg::with_name("task-name")
                        .index(1)
                        .required(true)
                        .multiple(true)
                        .help("Name of the task to add"),
                ),
        )
        .get_matches()
}

fn run_app(matches: ArgMatches) -> Result<(), String> {
    let taxfile_path_getter = &TaxfilePathGetterReal {
        get_env: env_getter_real,
        get_home: home_getter_real,
    };

    let user_cmd_runner = &UserCmdRunnerReal {
        taxfile_path_getter: taxfile_path_getter,
        get_env: env_getter_real,
    };

    let file_path = taxfile_path_getter.get_taxfile_path()?;
    let content_handler = &ContentHandlerReal { path: file_path };

    let outputer = &mut StringOutputerReal {};

    let task_formatter = &TaskFormatter {
        supports_colors: SHOULD_COLORIZE.should_colorize(),
    };

    match matches.subcommand() {
        (_, None) => cmd_list::cmd(outputer, content_handler, task_formatter),
        ("edit", _) => cmd_edit::cmd(taxfile_path_getter, user_cmd_runner),
        ("focus", Some(info)) => cmd_focus::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            value_t!(info.value_of("task-index"), usize).unwrap(),
            true,
        ),
        ("blur", Some(info)) => cmd_focus::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            value_t!(info.value_of("task-index"), usize).unwrap(),
            false,
        ),

        ("check", Some(info)) => cmd_check::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            value_t!(info.value_of("task-index"), usize).unwrap(),
            true,
        ),
        ("uncheck", Some(info)) => cmd_check::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            value_t!(info.value_of("task-index"), usize).unwrap(),
            false,
        ),

        ("list", _) => cmd_list::cmd(outputer, content_handler, task_formatter),
        ("current", _) => cmd_current::cmd(outputer, content_handler, task_formatter, false),
        ("cycle", _) => cmd_current::cmd(outputer, content_handler, task_formatter, true),

        ("prune", _) => cmd_prune::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
        ),

        ("cat", _) => cmd_cat::cmd(outputer, content_handler),

        ("which", _) => cmd_which::cmd(outputer, taxfile_path_getter),

        ("add", Some(info)) => cmd_add::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            info.values_of_lossy("task-name").unwrap(),
            cmd_add::AddPosition::Prepend,
        ),

        ("append", Some(info)) => cmd_add::cmd(
            outputer,
            content_handler,
            content_handler,
            user_cmd_runner,
            task_formatter,
            info.values_of_lossy("task-name").unwrap(),
            cmd_add::AddPosition::Append,
        ),
        _ => Err(format!("Unknown command")),
    }
}
