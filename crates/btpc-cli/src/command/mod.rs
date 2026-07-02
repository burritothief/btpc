mod parsers;

use parsers::{
    parse_creation_date, parse_entropy, parse_file_attributes, parse_node, parse_piece_length,
};
use std::path::PathBuf;

use btpc_core::create::CreateMode;
use clap::error::ErrorKind;
use clap::{ArgAction, Args, CommandFactory as _, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "btpc", version, about = "BitTorrent metainfo toolkit")]
pub(crate) struct Cli {
    /// Use this configuration file when configuration loading is enabled.
    #[arg(long, global = true, value_name = "PATH", conflicts_with = "no_config")]
    pub(crate) config: Option<PathBuf>,
    /// Disable implicit and environment-selected configuration.
    #[arg(long, global = true)]
    pub(crate) no_config: bool,
    /// Control colored terminal output.
    #[arg(long, global = true, value_enum)]
    pub(crate) color: Option<CliColorPolicy>,
    /// Increase diagnostic verbosity; may be repeated.
    #[arg(
        short = 'v',
        long,
        global = true,
        action = ArgAction::Count,
        conflicts_with = "quiet"
    )]
    pub(crate) verbose: u8,
    /// Suppress human summaries, warnings, and progress.
    #[arg(short = 'q', long, global = true)]
    pub(crate) quiet: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    pub(crate) fn validate(&self) -> Result<(), clap::Error> {
        let pretty = match &self.command {
            Command::Create(arguments) => arguments.pretty,
            Command::Inspect(arguments) => arguments.pretty,
            Command::Validate(arguments) => arguments.pretty,
            Command::Verify(arguments) => arguments.pretty,
            Command::Edit(arguments) => arguments.diff,
            Command::Magnet(_)
            | Command::Config(_)
            | Command::Completion(_)
            | Command::Completions(_)
            | Command::Manpage => false,
        };
        if self.quiet && pretty {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "the argument '--quiet' cannot be used with '--pretty'",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Create canonical v1, v2, or hybrid metainfo from a file or directory.
    Create(Box<CreateArgs>),
    /// Inspect validated metainfo without reading payload files.
    Inspect(InspectArgs),
    /// Validate metainfo structure without reading payload files.
    Validate(ValidateArgs),
    /// Verify payload files against metainfo hashes.
    Verify(VerifyArgs),
    /// Edit metainfo metadata without reading payload files.
    Edit(EditArgs),
    /// Print a deterministic magnet URI.
    Magnet(MagnetArgs),
    /// Locate, inspect, validate, and update configuration.
    Config(ConfigArgs),
    /// Generate, install, or uninstall shell completions.
    Completion(CompletionCommandArgs),
    /// Generate a shell completion script on stdout (deprecated).
    #[command(hide = true)]
    Completions(CompletionArgs),
    /// Generate the btpc(1) manual page on stdout.
    Manpage,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    /// Print the selected configuration path.
    Path,
    /// Create a minimal configuration file.
    Init {
        /// Replace an existing file.
        #[arg(long)]
        force: bool,
    },
    /// Print configuration with secrets redacted by default.
    Show {
        /// Validate and print the parsed deterministic representation.
        #[arg(long)]
        resolved: bool,
        /// Reveal configured secrets.
        #[arg(long)]
        show_secrets: bool,
        /// Emit JSON instead of TOML.
        #[arg(long)]
        json: bool,
    },
    /// Validate schema, references, cycles, conflicts, and permissions.
    Check,
    /// Explain resolved command values without executing the command.
    Explain(ConfigExplainArgs),
    /// Manage named tracker aliases.
    Tracker(ConfigTrackerArgs),
    /// Manage named creation presets.
    Preset(Box<ConfigPresetArgs>),
}

#[derive(Debug, Args)]
pub(crate) struct ConfigExplainArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigExplainCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigExplainCommand {
    /// Explain effective create values and provenance.
    Create(Box<CreateArgs>),
}

#[derive(Debug, Args)]
pub(crate) struct ConfigTrackerArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigTrackerCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigTrackerCommand {
    /// List tracker aliases.
    List {
        /// Reveal tracker URLs.
        #[arg(long)]
        show_secrets: bool,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Add or replace a tracker alias.
    Add { name: String, url: String },
    /// Remove a tracker alias.
    Remove { name: String },
}

#[derive(Debug, Args)]
pub(crate) struct ConfigPresetArgs {
    #[command(subcommand)]
    pub(crate) command: ConfigPresetCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigPresetCommand {
    /// List preset names.
    List {
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Show one preset.
    Show {
        name: String,
        /// Reveal configured URLs.
        #[arg(long)]
        show_secrets: bool,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Save or replace a preset.
    Save(Box<PresetSaveArgs>),
    /// Remove a preset.
    Remove { name: String },
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct PresetSaveArgs {
    pub(crate) name: String,
    /// Parent preset; may be repeated.
    #[arg(long = "extends")]
    pub(crate) extends: Vec<String>,
    #[arg(long, value_enum)]
    pub(crate) mode: Option<CliCreateMode>,
    #[arg(long)]
    pub(crate) piece_length: Option<u64>,
    #[arg(long)]
    pub(crate) private: bool,
    #[arg(long)]
    pub(crate) source: Option<String>,
    #[arg(long)]
    pub(crate) comment: Option<String>,
    #[arg(long)]
    pub(crate) created_by: Option<String>,
    #[arg(long)]
    pub(crate) creation_date: Option<i64>,
    #[arg(long = "name")]
    pub(crate) name_override: Option<String>,
    #[arg(long)]
    pub(crate) exclude_hidden: bool,
    #[arg(long, value_enum)]
    pub(crate) symlinks: Option<CliSymlinkPolicy>,
    #[arg(long, value_enum)]
    pub(crate) special_files: Option<CliSpecialFilePolicy>,
    #[arg(long)]
    pub(crate) exclude_empty_files: bool,
    #[arg(long)]
    pub(crate) reject_empty_directories: bool,
    #[arg(long = "tracker")]
    pub(crate) trackers: Vec<String>,
    #[arg(long = "tracker-alias")]
    pub(crate) tracker_aliases: Vec<String>,
    #[arg(long = "tracker-group")]
    pub(crate) tracker_groups: Vec<String>,
    #[arg(long = "web-seed")]
    pub(crate) web_seeds: Vec<String>,
    #[arg(long = "include")]
    pub(crate) includes: Vec<String>,
    #[arg(long = "exclude")]
    pub(crate) excludes: Vec<String>,
    #[arg(long)]
    pub(crate) threads: Option<usize>,
}

#[derive(Debug, Args)]
pub(crate) struct CompletionArgs {
    /// Shell whose completion syntax should be generated.
    #[arg(value_enum)]
    pub(crate) shell: Shell,
}

#[derive(Debug, Args)]
pub(crate) struct CompletionCommandArgs {
    #[command(subcommand)]
    pub(crate) command: CompletionCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CompletionCommand {
    /// Generate a shell completion script on stdout.
    Generate(CompletionArgs),
    /// Install shell completions in the standard per-user directory.
    Install(CompletionMutationArgs),
    /// Remove BTPC-generated shell completions.
    Uninstall(CompletionMutationArgs),
}

#[derive(Debug, Args)]
pub(crate) struct CompletionMutationArgs {
    /// Shell to install; detected from environment hints when omitted.
    #[arg(value_enum)]
    pub(crate) shell: Option<Shell>,
    /// Print the target and generated content without changing files.
    #[arg(long)]
    pub(crate) dry_run: bool,
    /// Replace an unrelated existing completion file.
    #[arg(long)]
    pub(crate) force: bool,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct InspectArgs {
    /// Metainfo file to read.
    pub(crate) input: PathBuf,
    /// Emit versioned JSON to stdout.
    #[arg(long)]
    pub(crate) json: bool,
    /// Select a field; may be repeated.
    #[arg(long = "field", value_enum)]
    pub(crate) fields: Vec<InspectField>,
    /// Include the flat file listing.
    #[arg(long)]
    pub(crate) files: bool,
    /// Render files as a deterministic tree.
    #[arg(long, conflicts_with = "files")]
    pub(crate) tree: bool,
    /// Encode raw torrent paths.
    #[arg(long, value_enum, default_value_t)]
    pub(crate) path_encoding: PathEncoding,
    /// Skip this many file rows.
    #[arg(long, default_value_t = 0)]
    pub(crate) offset: usize,
    /// Limit returned file rows.
    #[arg(long)]
    pub(crate) limit: Option<usize>,
    /// Select output representation.
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<CliOutputFormat>,
    /// Use the expanded human renderer.
    #[arg(long, conflicts_with_all = ["json", "quiet"])]
    pub(crate) pretty: bool,
    #[command(flatten)]
    pub(crate) limits: ReadLimitArgs,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct ValidateArgs {
    pub(crate) input: PathBuf,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(long, value_enum, conflicts_with = "json")]
    pub(crate) format: Option<ValidateFormat>,
    #[arg(long)]
    pub(crate) canonical: bool,
    #[arg(long)]
    pub(crate) warnings_as_errors: bool,
    #[arg(long, conflicts_with_all = ["json", "quiet"])]
    pub(crate) pretty: bool,
    #[command(flatten)]
    pub(crate) limits: ReadLimitArgs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum InspectField {
    Mode,
    Name,
    TotalSize,
    PieceLength,
    PieceCount,
    FileCount,
    HashV1,
    HashV2,
    Private,
    Trackers,
    WebSeeds,
    Nodes,
    Comment,
    Creator,
    CreationDate,
    Source,
    Canonicality,
    Warnings,
    Files,
    UnknownFields,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum PathEncoding {
    #[default]
    Utf8,
    Escaped,
    Hex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CliOutputFormat {
    Human,
    Plain,
    Json,
    JsonPretty,
    Tsv,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ValidateFormat {
    Human,
    Json,
    JsonPretty,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_field_names)]
pub(crate) struct ReadLimitArgs {
    /// Maximum metainfo bytes accepted while loading.
    #[arg(long)]
    pub(crate) max_input_bytes: Option<usize>,
    /// Maximum estimated owned allocation while loading.
    #[arg(long)]
    pub(crate) max_owned_bytes: Option<usize>,
    /// Maximum decimal digits accepted in one bencode integer.
    #[arg(long)]
    pub(crate) max_integer_digits: Option<usize>,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct VerifyArgs {
    /// Metainfo file to read.
    pub(crate) torrent: PathBuf,
    /// Payload file or directory to verify.
    pub(crate) payload: PathBuf,
    /// Stop after the first deterministic mismatch.
    #[arg(long)]
    pub(crate) fail_fast: bool,
    /// Report regular files absent from metainfo.
    #[arg(long)]
    pub(crate) extra_files: bool,
    /// Emit versioned JSON to stdout.
    #[arg(long)]
    pub(crate) json: bool,
    /// Use the expanded human renderer.
    #[arg(long, conflicts_with_all = ["json", "quiet"])]
    pub(crate) pretty: bool,
    #[command(flatten)]
    pub(crate) limits: ReadLimitArgs,
}

#[derive(Debug, Args)]
pub(crate) struct MagnetArgs {
    /// Metainfo file to read.
    pub(crate) input: PathBuf,
    /// Omit the display name parameter.
    #[arg(long)]
    pub(crate) no_display_name: bool,
    /// Omit tracker parameters.
    #[arg(long)]
    pub(crate) no_trackers: bool,
    /// Omit web seed parameters.
    #[arg(long)]
    pub(crate) no_web_seeds: bool,
    #[command(flatten)]
    pub(crate) limits: ReadLimitArgs,
}

#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct EditArgs {
    pub(crate) input: PathBuf,
    #[arg(short, long, conflicts_with = "in_place")]
    pub(crate) output: Option<PathBuf>,
    #[arg(long)]
    pub(crate) in_place: bool,
    #[arg(short, long)]
    pub(crate) force: bool,
    #[arg(long)]
    pub(crate) durable: bool,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long)]
    pub(crate) diff: bool,
    #[arg(long)]
    pub(crate) json: bool,
    #[arg(short = 'a', long = "tracker")]
    pub(crate) trackers: Vec<String>,
    #[arg(long = "tracker-alias")]
    pub(crate) tracker_aliases: Vec<String>,
    #[arg(long = "tracker-group")]
    pub(crate) tracker_groups: Vec<String>,
    #[arg(long)]
    pub(crate) clear_trackers: bool,
    #[arg(long = "web-seed")]
    pub(crate) web_seeds: Vec<String>,
    #[arg(long)]
    pub(crate) clear_web_seeds: bool,
    #[arg(long = "node", value_parser = parse_node)]
    pub(crate) nodes: Vec<(Vec<u8>, u16)>,
    #[arg(long)]
    pub(crate) clear_nodes: bool,
    #[arg(long, conflicts_with = "clear_comment")]
    pub(crate) comment: Option<String>,
    #[arg(long)]
    pub(crate) clear_comment: bool,
    #[arg(long, conflicts_with = "clear_created_by")]
    pub(crate) created_by: Option<String>,
    #[arg(long)]
    pub(crate) clear_created_by: bool,
    #[arg(long, conflicts_with = "clear_creation_date")]
    pub(crate) creation_date: Option<i64>,
    #[arg(long)]
    pub(crate) clear_creation_date: bool,
    #[arg(long, conflicts_with_all = ["public", "clear_private"])]
    pub(crate) private: bool,
    #[arg(long, conflicts_with = "clear_private")]
    pub(crate) public: bool,
    #[arg(long)]
    pub(crate) clear_private: bool,
    #[arg(long, conflicts_with = "clear_source")]
    pub(crate) source: Option<String>,
    #[arg(long)]
    pub(crate) clear_source: bool,
    #[arg(long = "file-attributes", value_parser = parse_file_attributes)]
    pub(crate) file_attributes: Vec<(Vec<Vec<u8>>, Vec<u8>)>,
}

#[derive(Clone, Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct CreateArgs {
    /// Payload files or directories.
    #[arg(num_args = 0.., required_unless_present = "batch")]
    pub(crate) inputs: Vec<PathBuf>,
    /// Versioned TOML batch manifest.
    #[arg(long, conflicts_with = "inputs")]
    pub(crate) batch: Option<PathBuf>,
    /// Torrent protocol representation.
    #[arg(long, value_enum)]
    pub(crate) mode: Option<CliCreateMode>,
    /// Destination .torrent path (defaults beside the payload).
    #[arg(short, long)]
    pub(crate) output: Option<PathBuf>,
    #[arg(long, conflicts_with = "output")]
    pub(crate) output_dir: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    pub(crate) jobs: usize,
    #[arg(long)]
    pub(crate) fail_fast: bool,
    /// Replace an existing destination.
    #[arg(short, long)]
    pub(crate) force: bool,
    /// Sync the destination directory after atomic publication where supported.
    #[arg(long)]
    pub(crate) durable: bool,
    /// Apply a named creation preset; may be repeated.
    #[arg(long = "preset")]
    pub(crate) presets: Vec<String>,
    /// Explicit piece length in bytes.
    #[arg(long, value_parser = parse_piece_length)]
    pub(crate) piece_length: Option<u64>,
    #[arg(long)]
    pub(crate) target_pieces: Option<u64>,
    #[arg(long, value_parser = parse_piece_length, requires = "target_pieces")]
    pub(crate) max_piece_length: Option<u64>,
    /// Add a tracker as its own tier; may be repeated.
    #[arg(short = 'a', long = "tracker")]
    pub(crate) trackers: Vec<String>,
    /// Clear configured and preset trackers before CLI additions.
    #[arg(long)]
    pub(crate) clear_trackers: bool,
    /// Add one comma-separated tracker tier; may be repeated.
    #[arg(long = "tracker-tier", value_delimiter = ',')]
    pub(crate) tracker_tier: Vec<String>,
    #[arg(long = "tracker-alias")]
    pub(crate) tracker_aliases: Vec<String>,
    #[arg(long = "tracker-group")]
    pub(crate) tracker_groups: Vec<String>,
    /// Add a web seed URL; may be repeated.
    #[arg(long = "web-seed")]
    pub(crate) web_seeds: Vec<String>,
    /// Clear configured and preset web seeds before CLI additions.
    #[arg(long)]
    pub(crate) clear_web_seeds: bool,
    /// Add a DHT node as HOST:PORT; may be repeated.
    #[arg(long = "node", value_parser = parse_node)]
    pub(crate) nodes: Vec<(Vec<u8>, u16)>,
    /// Set the private flag.
    #[arg(long, conflicts_with = "public")]
    pub(crate) private: bool,
    #[arg(long)]
    pub(crate) public: bool,
    /// Set the source field.
    #[arg(long)]
    pub(crate) source: Option<String>,
    #[arg(long, conflicts_with = "source")]
    pub(crate) clear_source: bool,
    /// Set the top-level comment.
    #[arg(long)]
    pub(crate) comment: Option<String>,
    #[arg(long, conflicts_with = "comment")]
    pub(crate) clear_comment: bool,
    /// Set the creator string.
    #[arg(long, conflicts_with = "clear_created_by")]
    pub(crate) created_by: Option<String>,
    /// Omit the creator string instead of using the versioned default.
    #[arg(
        long = "no-created-by",
        alias = "clear-created-by",
        conflicts_with = "created_by"
    )]
    pub(crate) clear_created_by: bool,
    /// Include an explicit Unix creation timestamp.
    #[arg(long, value_parser = parse_creation_date)]
    pub(crate) creation_date: Option<CreationDateValue>,
    #[arg(long, value_parser = parse_entropy)]
    pub(crate) entropy: Option<EntropyValue>,
    /// Override the torrent root name.
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Exclude dot-prefixed files and directories.
    #[arg(long)]
    pub(crate) exclude_hidden: bool,
    /// Symbolic-link policy.
    #[arg(long, value_enum)]
    pub(crate) symlinks: Option<CliSymlinkPolicy>,
    /// Special-file policy.
    #[arg(long, value_enum)]
    pub(crate) special_files: Option<CliSpecialFilePolicy>,
    /// Exclude zero-length files.
    #[arg(long)]
    pub(crate) exclude_empty_files: bool,
    /// Reject empty directories instead of ignoring them.
    #[arg(long)]
    pub(crate) reject_empty_directories: bool,
    /// Include only paths matching this glob; may be repeated.
    #[arg(long = "include")]
    pub(crate) includes: Vec<String>,
    /// Clear configured and preset include patterns before CLI additions.
    #[arg(long)]
    pub(crate) clear_includes: bool,
    /// Exclude paths matching this glob; may be repeated.
    #[arg(long = "exclude")]
    pub(crate) excludes: Vec<String>,
    /// Clear configured and preset exclude patterns before CLI additions.
    #[arg(long)]
    pub(crate) clear_excludes: bool,
    /// v1 hashing threads; 0 selects a conservative automatic count, 1 is sequential.
    #[arg(long)]
    pub(crate) threads: Option<usize>,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long = "print", value_enum, conflicts_with = "json")]
    pub(crate) print: Vec<CreatePrint>,
    /// Emit a versioned JSON result to stdout.
    #[arg(long)]
    pub(crate) json: bool,
    /// Use the expanded human completion renderer.
    #[arg(long, conflicts_with_all = ["json", "quiet"])]
    pub(crate) pretty: bool,
}

#[derive(Clone, Debug)]
// Spec: CLI-CREATE-002
pub(crate) enum EntropyValue {
    None,
    Exact(Vec<u8>),
    Random,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CreationDateValue {
    None,
    Timestamp(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CreatePrint {
    Path,
    InfoHashV1,
    InfoHashV2,
    Magnet,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CliColorPolicy {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CliSymlinkPolicy {
    #[default]
    Reject,
    Skip,
    Follow,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CliSpecialFilePolicy {
    #[default]
    Reject,
    Skip,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CliCreateMode {
    #[default]
    V1,
    V2,
    Hybrid,
}

impl From<CliCreateMode> for CreateMode {
    fn from(mode: CliCreateMode) -> Self {
        match mode {
            CliCreateMode::V1 => Self::V1,
            CliCreateMode::V2 => Self::V2,
            CliCreateMode::Hybrid => Self::Hybrid,
        }
    }
}
