#![deny(warnings)]

use clap::{Parser, Subcommand};
use colored::control::SHOULD_COLORIZE;

mod services;
use services::{
    ContentHandlerReal, StringOutputerReal, TaskFormatter, TaxfilePathGetter,
    TaxfilePathGetterReal, UserCmdRunnerReal, env_getter_real, home_getter_real,
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
mod cmd_focus_section;
mod cmd_list;
mod cmd_prune;
mod cmd_which;

#[derive(Parser)]
#[command(name = "Tax", version, about = "CLI Task List Manager")]
struct Cli {
    /// Print all open tasks regardless of section focus
    #[arg(short, long)]
    all: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Edit the current task list in $EDITOR
    Edit,

    /// Focus the given task
    Focus {
        /// Number of the task to focus
        task_index: String,
    },

    /// Blur the given task
    #[command(alias = "unfocus")]
    Blur {
        /// Number of the task to blur
        task_index: String,
    },

    /// Mark the given task as completed
    Check {
        /// Number of the task to mark as completed
        task_index: usize,
    },

    /// Mark the given task as not completed
    Uncheck {
        /// Number of the task to mark as not completed
        task_index: usize,
    },

    /// Print all open tasks of the list, or of the focused section if any
    #[command(alias = "ls")]
    List {
        /// Print all open tasks regardless of section focus
        #[arg(short, long)]
        all: bool,
    },

    /// Print the first open (focused if any) task of the list
    Current,

    /// Like current, but changes task every minute if no task is focused
    Cycle,

    /// Remove all completed tasks from the task list
    #[command(alias = "purge")]
    Prune,

    /// Print the content of the task file without any processing
    #[command(alias = "view")]
    Cat,

    /// Print the path of the current task list file
    Which,

    /// Add the given task at the top of the task list
    #[command(alias = "push", alias = "prepend")]
    Add {
        /// Section where to add task
        #[arg(short, long)]
        section: Option<String>,

        /// Name of the task to add
        #[arg(required = true, num_args = 1..)]
        task_name: Vec<String>,
    },

    /// Add the given task at the bottom of the task list
    Append {
        /// Section where to add task
        #[arg(short, long)]
        section: Option<String>,

        /// Name of the task to add
        #[arg(required = true, num_args = 1..)]
        task_name: Vec<String>,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    run_app(cli)
}

fn run_app(cli: Cli) -> Result<(), String> {
    let taxfile_path_getter = &TaxfilePathGetterReal {
        get_env: env_getter_real,
        get_home: home_getter_real,
    };

    let user_cmd_runner = &UserCmdRunnerReal {
        taxfile_path_getter,
        get_env: env_getter_real,
    };

    let file_path = taxfile_path_getter.get_taxfile_path()?;

    let content_handler_ref = &ContentHandlerReal {
        path: file_path.clone(),
    };
    let content_handler_mutref = &mut ContentHandlerReal { path: file_path };

    let outputer = &mut StringOutputerReal {};

    let task_formatter = &TaskFormatter {
        supports_colors: SHOULD_COLORIZE.should_colorize(),
    };

    let all = cli.all;
    if all {
        match &cli.command {
            None | Some(Commands::List { .. }) => (),
            _ => return Err(String::from("-a, --all not implemented on this command")),
        };
    }

    match cli.command {
        None => cmd_list::cmd(outputer, content_handler_ref, task_formatter, all),
        Some(Commands::Edit) => cmd_edit::cmd(taxfile_path_getter, user_cmd_runner),
        Some(Commands::Focus { task_index }) => match task_index.parse::<usize>() {
            Ok(rank_one_based) => cmd_focus::cmd(
                outputer,
                content_handler_ref,
                content_handler_mutref,
                user_cmd_runner,
                task_formatter,
                rank_one_based,
                true,
            ),
            Err(_) => cmd_focus_section::cmd(
                outputer,
                content_handler_ref,
                content_handler_mutref,
                user_cmd_runner,
                task_index,
                true,
            ),
        },
        Some(Commands::Blur { task_index }) => match task_index.parse::<usize>() {
            Ok(rank_one_based) => cmd_focus::cmd(
                outputer,
                content_handler_ref,
                content_handler_mutref,
                user_cmd_runner,
                task_formatter,
                rank_one_based,
                false,
            ),
            Err(_) => cmd_focus_section::cmd(
                outputer,
                content_handler_ref,
                content_handler_mutref,
                user_cmd_runner,
                task_index,
                false,
            ),
        },
        Some(Commands::Check { task_index }) => cmd_check::cmd(
            outputer,
            content_handler_ref,
            content_handler_mutref,
            user_cmd_runner,
            task_formatter,
            task_index,
            true,
        ),
        Some(Commands::Uncheck { task_index }) => cmd_check::cmd(
            outputer,
            content_handler_ref,
            content_handler_mutref,
            user_cmd_runner,
            task_formatter,
            task_index,
            false,
        ),
        Some(Commands::List { all: list_all }) => {
            cmd_list::cmd(outputer, content_handler_ref, task_formatter, list_all)
        }
        Some(Commands::Current) => {
            cmd_current::cmd(outputer, content_handler_ref, task_formatter, false)
        }
        Some(Commands::Cycle) => {
            cmd_current::cmd(outputer, content_handler_ref, task_formatter, true)
        }
        Some(Commands::Prune) => cmd_prune::cmd(
            outputer,
            content_handler_ref,
            content_handler_mutref,
            user_cmd_runner,
            task_formatter,
        ),
        Some(Commands::Cat) => cmd_cat::cmd(outputer, content_handler_ref),
        Some(Commands::Which) => cmd_which::cmd(outputer, taxfile_path_getter),
        Some(Commands::Add { section, task_name }) => cmd_add::cmd(
            outputer,
            content_handler_ref,
            content_handler_mutref,
            user_cmd_runner,
            task_formatter,
            task_name,
            section,
            cmd_add::AddPosition::Prepend,
        ),
        Some(Commands::Append { section, task_name }) => cmd_add::cmd(
            outputer,
            content_handler_ref,
            content_handler_mutref,
            user_cmd_runner,
            task_formatter,
            task_name,
            section,
            cmd_add::AddPosition::Append,
        ),
    }
}
