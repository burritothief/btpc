mod command;
mod config;
mod context;
mod diagnostics;
mod handlers;
mod output;
mod progress;
mod reference;
mod render;

use std::process::ExitCode;
use std::{env, ffi::OsStr, path::PathBuf};

use btpc_core::Error;
use clap::Parser as _;

use crate::command::{Cli, Command, CompletionCommand};
use crate::context::ExecutionContext;

fn main() -> ExitCode {
    if let Some(destination) = markdown_destination() {
        return match reference::markdown(&destination) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let cli = Cli::parse();
    if let Err(error) = cli.validate() {
        error.exit();
    }
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => diagnostics::report(&error, cli.color),
    }
}

fn run(cli: &Cli) -> Result<(), Error> {
    if let Command::Config(arguments) = &cli.command {
        return config::run_command(cli, arguments);
    }
    let configuration = config::Configuration::load(cli)?;
    let context = ExecutionContext::from_cli(cli, configuration.global());
    match &cli.command {
        Command::Create(arguments) => handlers::create(arguments, &context, &configuration),
        Command::Inspect(arguments) => handlers::inspect(arguments, &context),
        Command::Validate(arguments) => handlers::validate(arguments, &context),
        Command::Verify(arguments) => handlers::verify(arguments, &context),
        Command::Edit(arguments) => handlers::edit(arguments, &context, &configuration),
        Command::Magnet(arguments) => handlers::magnet(arguments),
        Command::Config(_) => unreachable!("config commands dispatch before loading"),
        Command::Completion(arguments) => match &arguments.command {
            CompletionCommand::Generate(arguments) => reference::generate(arguments.shell),
            CompletionCommand::Install(arguments) => reference::install(arguments),
            CompletionCommand::Uninstall(arguments) => reference::uninstall(arguments),
        },
        Command::Completions(arguments) => {
            if !cli.quiet {
                eprintln!(
                    "warning: `btpc completions` is deprecated; use `btpc completion generate`"
                );
            }
            reference::generate(arguments.shell)
        }
        Command::Manpage => reference::manpage(),
    }
}

fn markdown_destination() -> Option<PathBuf> {
    let mut arguments = env::args_os();
    arguments.next();
    if arguments.next().as_deref() != Some(OsStr::new("__generate-markdown")) {
        return None;
    }
    let destination = arguments.next().unwrap_or_else(|| {
        eprintln!("error: __generate-markdown requires a destination");
        std::process::exit(2);
    });
    if arguments.next().is_some() {
        eprintln!("error: __generate-markdown accepts one destination");
        std::process::exit(2);
    }
    Some(destination.into())
}
