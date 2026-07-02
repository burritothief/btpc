use std::io::IsTerminal as _;
use std::path::{Path, PathBuf};

use btpc_core::create::CancellationToken;

use crate::command::{Cli, CliColorPolicy, Command};
use crate::config::GlobalConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub(crate) enum OutputMode {
    Human,
    Plain,
    Json,
    JsonPretty,
    Tsv,
}

impl OutputMode {
    pub(crate) const fn is_machine(self) -> bool {
        !matches!(self, Self::Human)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorPolicy {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgressPolicy {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OutputEnvironment {
    no_color: bool,
    stderr_is_terminal: bool,
    machine_output: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConfigSelection {
    Implicit,
    Explicit(PathBuf),
    Disabled,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ExecutionContext {
    output_mode: OutputMode,
    pretty: bool,
    quiet: bool,
    verbosity: u8,
    color_policy: ColorPolicy,
    progress_policy: ProgressPolicy,
    cancellation: CancellationToken,
    config: ConfigSelection,
    config_provenance: Option<String>,
}

impl ExecutionContext {
    pub(crate) fn from_cli(cli: &Cli, global: &GlobalConfig) -> Self {
        let (output_mode, pretty) = match &cli.command {
            Command::Create(arguments) => (output_mode(arguments.json), arguments.pretty),
            Command::Inspect(arguments) => (
                arguments
                    .format
                    .map_or_else(|| output_mode(arguments.json), inspect_output_mode),
                arguments.pretty,
            ),
            Command::Validate(arguments) => (
                arguments
                    .format
                    .map_or_else(|| output_mode(arguments.json), validate_output_mode),
                arguments.pretty,
            ),
            Command::Verify(arguments) => (output_mode(arguments.json), arguments.pretty),
            Command::Edit(arguments) => (OutputMode::Human, arguments.diff),
            Command::Magnet(_)
            | Command::Config(_)
            | Command::Completion(_)
            | Command::Completions(_)
            | Command::Manpage => (OutputMode::Human, false),
        };
        let no_color = std::env::var_os("NO_COLOR").is_some();
        let stderr_is_terminal = std::io::stderr().is_terminal();
        let environment = OutputEnvironment {
            no_color,
            stderr_is_terminal,
            machine_output: output_mode.is_machine(),
        };
        let quiet = cli.quiet || global.quiet.unwrap_or(false);
        let verbosity = if cli.verbose > 0 {
            cli.verbose
        } else {
            global.verbose.unwrap_or(0)
        };
        let color_policy =
            resolve_color(cli.color.or(global.color).unwrap_or_default(), environment);
        let progress_policy = resolve_progress(quiet, environment);
        Self {
            output_mode,
            pretty,
            quiet,
            verbosity,
            color_policy,
            progress_policy,
            cancellation: CancellationToken::new(),
            config: if cli.no_config {
                ConfigSelection::Disabled
            } else if let Some(path) = &cli.config {
                ConfigSelection::Explicit(path.clone())
            } else {
                ConfigSelection::Implicit
            },
            config_provenance: None,
        }
    }

    pub(crate) const fn output_mode(&self) -> OutputMode {
        self.output_mode
    }

    #[allow(dead_code)]
    pub(crate) const fn pretty(&self) -> bool {
        self.pretty
    }

    pub(crate) fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    #[allow(dead_code)]
    pub(crate) const fn verbosity(&self) -> u8 {
        self.verbosity
    }

    #[allow(dead_code)]
    pub(crate) const fn color_policy(&self) -> ColorPolicy {
        self.color_policy
    }

    #[allow(dead_code)]
    pub(crate) const fn progress_policy(&self) -> ProgressPolicy {
        self.progress_policy
    }

    #[allow(dead_code)]
    pub(crate) fn config(&self) -> &ConfigSelection {
        &self.config
    }

    #[allow(dead_code)]
    pub(crate) fn config_provenance(&self) -> Option<&str> {
        self.config_provenance.as_deref()
    }

    pub(crate) const fn human_output_enabled(&self) -> bool {
        !self.quiet && matches!(self.output_mode, OutputMode::Human)
    }
}

const fn inspect_output_mode(format: crate::command::CliOutputFormat) -> OutputMode {
    match format {
        crate::command::CliOutputFormat::Human => OutputMode::Human,
        crate::command::CliOutputFormat::Plain => OutputMode::Plain,
        crate::command::CliOutputFormat::Json => OutputMode::Json,
        crate::command::CliOutputFormat::JsonPretty => OutputMode::JsonPretty,
        crate::command::CliOutputFormat::Tsv => OutputMode::Tsv,
    }
}

const fn validate_output_mode(format: crate::command::ValidateFormat) -> OutputMode {
    match format {
        crate::command::ValidateFormat::Human => OutputMode::Human,
        crate::command::ValidateFormat::Json => OutputMode::Json,
        crate::command::ValidateFormat::JsonPretty => OutputMode::JsonPretty,
    }
}

const fn output_mode(json: bool) -> OutputMode {
    if json {
        OutputMode::Json
    } else {
        OutputMode::Human
    }
}

const fn resolve_color(requested: CliColorPolicy, environment: OutputEnvironment) -> ColorPolicy {
    match requested {
        CliColorPolicy::Always => ColorPolicy::Enabled,
        CliColorPolicy::Never => ColorPolicy::Disabled,
        CliColorPolicy::Auto => {
            if environment.no_color || !environment.stderr_is_terminal || environment.machine_output
            {
                ColorPolicy::Disabled
            } else {
                ColorPolicy::Enabled
            }
        }
    }
}

const fn resolve_progress(quiet: bool, environment: OutputEnvironment) -> ProgressPolicy {
    if quiet
        || environment.no_color
        || !environment.stderr_is_terminal
        || environment.machine_output
    {
        ProgressPolicy::Disabled
    } else {
        ProgressPolicy::Enabled
    }
}

#[allow(dead_code)]
pub(crate) fn config_path(selection: &ConfigSelection) -> Option<&Path> {
    match selection {
        ConfigSelection::Explicit(path) => Some(path),
        ConfigSelection::Implicit | ConfigSelection::Disabled => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorPolicy, OutputEnvironment, ProgressPolicy, resolve_color, resolve_progress};
    use crate::command::CliColorPolicy;

    #[test]
    fn explicit_color_precedes_environment_terminal_and_machine_state() {
        assert_eq!(
            resolve_color(
                CliColorPolicy::Always,
                OutputEnvironment {
                    no_color: true,
                    stderr_is_terminal: false,
                    machine_output: true,
                },
            ),
            ColorPolicy::Enabled
        );
        assert_eq!(
            resolve_color(
                CliColorPolicy::Never,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            ColorPolicy::Disabled
        );
        assert_eq!(
            resolve_color(
                CliColorPolicy::Auto,
                OutputEnvironment {
                    no_color: true,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            ColorPolicy::Disabled
        );
        assert_eq!(
            resolve_color(
                CliColorPolicy::Auto,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: false,
                    machine_output: false,
                },
            ),
            ColorPolicy::Disabled
        );
        assert_eq!(
            resolve_color(
                CliColorPolicy::Auto,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: true,
                },
            ),
            ColorPolicy::Disabled
        );
        assert_eq!(
            resolve_color(
                CliColorPolicy::Auto,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            ColorPolicy::Enabled
        );
    }

    #[test]
    fn progress_requires_interactive_human_output() {
        assert_eq!(
            resolve_progress(
                false,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            ProgressPolicy::Enabled
        );
        for state in [
            resolve_progress(
                true,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            resolve_progress(
                false,
                OutputEnvironment {
                    no_color: true,
                    stderr_is_terminal: true,
                    machine_output: false,
                },
            ),
            resolve_progress(
                false,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: false,
                    machine_output: false,
                },
            ),
            resolve_progress(
                false,
                OutputEnvironment {
                    no_color: false,
                    stderr_is_terminal: true,
                    machine_output: true,
                },
            ),
        ] {
            assert_eq!(state, ProgressPolicy::Disabled);
        }
    }
}
