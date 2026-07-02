mod path_policy;

use path_policy::{Environment, default_path, selected_path};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use btpc_core::Error;
use serde::{Deserialize, Serialize};

use crate::command::{
    Cli, CliColorPolicy, CliCreateMode, CliSpecialFilePolicy, CliSymlinkPolicy, ConfigArgs,
    ConfigCommand, ConfigExplainCommand, ConfigPresetCommand, ConfigTrackerCommand, CreateArgs,
    PresetSaveArgs,
};
use crate::diagnostics::suggestion;
use crate::output::{REDACTED_URL, stderr_line, stdout_line, stdout_text, write_json};

const CONFIG_VERSION: u32 = 1;

// Spec: CLI-CONFIG-001
// Spec: CLI-PRESET-001
// Spec: CLI-CONFIG-CMD-001

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct GlobalConfig {
    pub(crate) color: Option<CliColorPolicy>,
    pub(crate) verbose: Option<u8>,
    pub(crate) quiet: Option<bool>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct ConfigFile {
    version: u32,
    global: GlobalConfig,
    create: CreateValues,
    trackers: BTreeMap<String, TrackerDefinition>,
    tracker_groups: BTreeMap<String, TrackerGroup>,
    presets: BTreeMap<String, Preset>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TrackerDefinition {
    url: String,
}

impl fmt::Debug for TrackerDefinition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TrackerDefinition { url: <redacted> }")
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TrackerGroup {
    trackers: Vec<String>,
}

impl fmt::Debug for ConfigFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigFile")
            .field("version", &self.version)
            .field("global", &self.global)
            .field("create", &self.create)
            .field("tracker_names", &self.trackers.keys().collect::<Vec<_>>())
            .field("tracker_groups", &self.tracker_groups)
            .field("preset_names", &self.presets.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Preset {
    extends: Vec<String>,
    mode: Option<CliCreateMode>,
    piece_length: Option<u64>,
    trackers: Vec<String>,
    tracker_aliases: Vec<String>,
    tracker_groups: Vec<String>,
    web_seeds: Vec<String>,
    private: Option<bool>,
    source: Option<String>,
    comment: Option<String>,
    created_by: Option<String>,
    creation_date: Option<i64>,
    name: Option<String>,
    exclude_hidden: Option<bool>,
    symlinks: Option<CliSymlinkPolicy>,
    special_files: Option<CliSpecialFilePolicy>,
    exclude_empty_files: Option<bool>,
    reject_empty_directories: Option<bool>,
    includes: Vec<String>,
    excludes: Vec<String>,
    threads: Option<usize>,
}

impl fmt::Debug for Preset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Preset")
            .field("extends", &self.extends)
            .field("mode", &self.mode)
            .field("piece_length", &self.piece_length)
            .field("tracker_count", &self.trackers.len())
            .field("tracker_aliases", &self.tracker_aliases)
            .field("tracker_groups", &self.tracker_groups)
            .field("web_seed_count", &self.web_seeds.len())
            .field("private", &self.private)
            .field("source", &self.source.as_ref().map(|_| "<redacted>"))
            .field("comment", &self.comment.as_ref().map(|_| "<redacted>"))
            .field("created_by", &self.created_by)
            .field("creation_date", &self.creation_date)
            .field("name", &self.name)
            .field("exclude_hidden", &self.exclude_hidden)
            .field("symlinks", &self.symlinks)
            .field("special_files", &self.special_files)
            .field("exclude_empty_files", &self.exclude_empty_files)
            .field("reject_empty_directories", &self.reject_empty_directories)
            .field("includes", &self.includes)
            .field("excludes", &self.excludes)
            .field("threads", &self.threads)
            .finish()
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct CreateValues {
    mode: Option<CliCreateMode>,
    piece_length: Option<u64>,
    trackers: Vec<String>,
    tracker_aliases: Vec<String>,
    tracker_groups: Vec<String>,
    web_seeds: Vec<String>,
    private: Option<bool>,
    source: Option<String>,
    comment: Option<String>,
    created_by: Option<String>,
    creation_date: Option<i64>,
    name: Option<String>,
    exclude_hidden: Option<bool>,
    symlinks: Option<CliSymlinkPolicy>,
    special_files: Option<CliSpecialFilePolicy>,
    exclude_empty_files: Option<bool>,
    reject_empty_directories: Option<bool>,
    includes: Vec<String>,
    excludes: Vec<String>,
    threads: Option<usize>,
}

impl fmt::Debug for CreateValues {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateValues")
            .field("mode", &self.mode)
            .field("piece_length", &self.piece_length)
            .field("tracker_count", &self.trackers.len())
            .field("tracker_aliases", &self.tracker_aliases)
            .field("tracker_groups", &self.tracker_groups)
            .field("web_seed_count", &self.web_seeds.len())
            .field("private", &self.private)
            .field("source", &self.source.as_ref().map(|_| "<redacted>"))
            .field("comment", &self.comment.as_ref().map(|_| "<redacted>"))
            .field("created_by", &self.created_by)
            .field("creation_date", &self.creation_date)
            .field("name", &self.name)
            .field("exclude_hidden", &self.exclude_hidden)
            .field("symlinks", &self.symlinks)
            .field("special_files", &self.special_files)
            .field("exclude_empty_files", &self.exclude_empty_files)
            .field("reject_empty_directories", &self.reject_empty_directories)
            .field("includes", &self.includes)
            .field("excludes", &self.excludes)
            .field("threads", &self.threads)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Provenance {
    values: BTreeMap<&'static str, String>,
}

impl Provenance {
    fn defaults() -> Self {
        let values = [
            "mode",
            "piece_length",
            "trackers",
            "web_seeds",
            "private",
            "source",
            "comment",
            "created_by",
            "creation_date",
            "name",
            "exclude_hidden",
            "symlinks",
            "special_files",
            "exclude_empty_files",
            "reject_empty_directories",
            "includes",
            "excludes",
            "threads",
        ]
        .into_iter()
        .map(|field| (field, "default".to_owned()))
        .collect();
        Self { values }
    }

    fn set(&mut self, field: &'static str, source: &str) {
        self.values.insert(field, source.to_owned());
    }
}

#[derive(Clone, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct ResolvedCreate {
    pub(crate) mode: CliCreateMode,
    pub(crate) piece_length: Option<u64>,
    pub(crate) target_pieces: Option<u64>,
    pub(crate) max_piece_length: Option<u64>,
    pub(crate) trackers: Vec<Vec<String>>,
    pub(crate) web_seeds: Vec<String>,
    pub(crate) private: bool,
    pub(crate) private_explicit: bool,
    pub(crate) source: Option<String>,
    pub(crate) comment: Option<String>,
    pub(crate) created_by: Option<String>,
    pub(crate) omit_created_by: bool,
    pub(crate) creation_date: Option<i64>,
    pub(crate) entropy: Option<Vec<u8>>,
    pub(crate) name: Option<String>,
    pub(crate) exclude_hidden: bool,
    pub(crate) symlinks: CliSymlinkPolicy,
    pub(crate) special_files: CliSpecialFilePolicy,
    pub(crate) exclude_empty_files: bool,
    pub(crate) reject_empty_directories: bool,
    pub(crate) includes: Vec<String>,
    pub(crate) excludes: Vec<String>,
    pub(crate) threads: usize,
    pub(crate) provenance: Provenance,
}

impl Default for ResolvedCreate {
    fn default() -> Self {
        Self {
            mode: CliCreateMode::V1,
            piece_length: None,
            target_pieces: None,
            max_piece_length: None,
            trackers: Vec::new(),
            web_seeds: Vec::new(),
            private: false,
            private_explicit: false,
            source: None,
            comment: None,
            created_by: None,
            omit_created_by: false,
            creation_date: None,
            entropy: None,
            name: None,
            exclude_hidden: false,
            symlinks: CliSymlinkPolicy::Reject,
            special_files: CliSpecialFilePolicy::Reject,
            exclude_empty_files: false,
            reject_empty_directories: false,
            includes: Vec::new(),
            excludes: Vec::new(),
            threads: 0,
            provenance: Provenance::defaults(),
        }
    }
}

impl fmt::Debug for ResolvedCreate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResolvedCreate")
            .field("mode", &self.mode)
            .field("piece_length", &self.piece_length)
            .field("target_pieces", &self.target_pieces)
            .field("max_piece_length", &self.max_piece_length)
            .field("tracker_count", &self.trackers.len())
            .field("web_seed_count", &self.web_seeds.len())
            .field("private", &self.private)
            .field("private_explicit", &self.private_explicit)
            .field("source", &self.source.as_ref().map(|_| "<redacted>"))
            .field("comment", &self.comment.as_ref().map(|_| "<redacted>"))
            .field("created_by", &self.created_by)
            .field("omit_created_by", &self.omit_created_by)
            .field("creation_date", &self.creation_date)
            .field("entropy", &self.entropy.as_ref().map(|_| "<redacted>"))
            .field("name", &self.name)
            .field("exclude_hidden", &self.exclude_hidden)
            .field("symlinks", &self.symlinks)
            .field("special_files", &self.special_files)
            .field("exclude_empty_files", &self.exclude_empty_files)
            .field("reject_empty_directories", &self.reject_empty_directories)
            .field("includes", &self.includes)
            .field("excludes", &self.excludes)
            .field("threads", &self.threads)
            .field("provenance", &self.provenance)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Configuration {
    path: Option<PathBuf>,
    file: ConfigFile,
}

impl Configuration {
    pub(crate) fn resolve_tracker_tiers(
        &self,
        trackers: &[String],
        aliases: &[String],
        groups: &[String],
    ) -> Result<Vec<Vec<String>>, Error> {
        let mut tiers = trackers
            .iter()
            .cloned()
            .map(|value| vec![value])
            .collect::<Vec<_>>();
        for alias in aliases {
            tiers.push(vec![self.tracker(alias)?]);
        }
        for name in groups {
            let group = self.file.tracker_groups.get(name).ok_or_else(|| {
                Error::metainfo_field(
                    "config.tracker_groups",
                    format!("unknown tracker group {name:?}"),
                )
            })?;
            tiers.push(
                group
                    .trackers
                    .iter()
                    .map(|alias| self.tracker(alias))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(tiers)
    }
    pub(crate) fn load(cli: &Cli) -> Result<Self, Error> {
        let path = selected_path(cli);
        let Some(path) = path else {
            return Ok(Self {
                path: None,
                file: ConfigFile {
                    version: CONFIG_VERSION,
                    ..ConfigFile::default()
                },
            });
        };
        if !path.exists() {
            if cli.config.is_some() || env::var_os("BTPC_CONFIG").is_some() {
                return Err(Error::io(
                    &path,
                    std::io::Error::new(std::io::ErrorKind::NotFound, "configuration not found"),
                ));
            }
            return Ok(Self {
                path: None,
                file: ConfigFile {
                    version: CONFIG_VERSION,
                    ..ConfigFile::default()
                },
            });
        }
        let contents = fs::read_to_string(&path).map_err(|source| Error::io(&path, source))?;
        let file = toml::from_str::<ConfigFile>(&contents).map_err(|error| {
            let location = error
                .span()
                .map_or_else(String::new, |span| format!(" at byte {}", span.start));
            Error::metainfo_field(
                "config",
                format!("invalid configuration TOML{location}: {}", error.message()),
            )
        })?;
        if file.version != CONFIG_VERSION {
            return Err(Error::metainfo_field(
                "config.version",
                format!(
                    "unsupported configuration version {}; expected {CONFIG_VERSION}",
                    file.version
                ),
            ));
        }
        Ok(Self {
            path: Some(path),
            file,
        })
    }

    pub(crate) const fn global(&self) -> &GlobalConfig {
        &self.file.global
    }

    pub(crate) fn resolve_create(&self, arguments: &CreateArgs) -> Result<ResolvedCreate, Error> {
        let mut resolved = ResolvedCreate::default();
        self.apply_values(&mut resolved, &self.file.create, "config")?;
        for preset in &arguments.presets {
            self.apply_preset(&mut resolved, preset, &mut Vec::new())?;
        }
        apply_cli(&mut resolved, arguments);
        let tiers =
            self.resolve_tracker_tiers(&[], &arguments.tracker_aliases, &arguments.tracker_groups)?;
        append_unique_tiers(&mut resolved.trackers, tiers);
        Ok(resolved)
    }

    fn validate(&self) -> Result<(), Error> {
        let mut base = ResolvedCreate::default();
        self.apply_values(&mut base, &self.file.create, "config")?;
        validate_resolved(&base)?;
        for name in self.file.presets.keys() {
            let mut resolved = base.clone();
            self.apply_preset(&mut resolved, name, &mut Vec::new())?;
            validate_resolved(&resolved)?;
        }
        Ok(())
    }

    fn apply_preset(
        &self,
        resolved: &mut ResolvedCreate,
        name: &str,
        chain: &mut Vec<String>,
    ) -> Result<(), Error> {
        if let Some(position) = chain.iter().position(|entry| entry == name) {
            let mut cycle = chain[position..].to_vec();
            cycle.push(name.to_owned());
            return Err(Error::metainfo_field(
                "config.presets",
                format!("preset inheritance cycle: {}", cycle.join(" -> ")),
            ));
        }
        let preset = self.file.presets.get(name).ok_or_else(|| {
            let mut missing = chain.clone();
            missing.push(name.to_owned());
            let suggestion = suggestion(name, self.file.presets.keys().map(String::as_str))
                .map_or_else(String::new, |candidate| {
                    format!("; did you mean {candidate:?}?")
                });
            Error::metainfo_field(
                "config.presets",
                format!(
                    "missing preset in chain: {}{suggestion}",
                    missing.join(" -> ")
                ),
            )
        })?;
        chain.push(name.to_owned());
        for parent in &preset.extends {
            self.apply_preset(resolved, parent, chain)?;
        }
        chain.pop();
        let values = CreateValues {
            mode: preset.mode,
            piece_length: preset.piece_length,
            trackers: preset.trackers.clone(),
            tracker_aliases: preset.tracker_aliases.clone(),
            tracker_groups: preset.tracker_groups.clone(),
            web_seeds: preset.web_seeds.clone(),
            private: preset.private,
            source: preset.source.clone(),
            comment: preset.comment.clone(),
            created_by: preset.created_by.clone(),
            creation_date: preset.creation_date,
            name: preset.name.clone(),
            exclude_hidden: preset.exclude_hidden,
            symlinks: preset.symlinks,
            special_files: preset.special_files,
            exclude_empty_files: preset.exclude_empty_files,
            reject_empty_directories: preset.reject_empty_directories,
            includes: preset.includes.clone(),
            excludes: preset.excludes.clone(),
            threads: preset.threads,
        };
        self.apply_values(resolved, &values, &format!("preset:{name}"))
    }

    fn apply_values(
        &self,
        resolved: &mut ResolvedCreate,
        values: &CreateValues,
        source: &str,
    ) -> Result<(), Error> {
        macro_rules! scalar {
            ($field:ident) => {
                if let Some(value) = &values.$field {
                    resolved.$field = value.clone();
                    resolved.provenance.set(stringify!($field), source);
                }
            };
        }
        scalar!(mode);
        scalar!(private);
        if values.private.is_some() {
            resolved.private_explicit = true;
        }
        scalar!(exclude_hidden);
        scalar!(symlinks);
        scalar!(special_files);
        scalar!(exclude_empty_files);
        scalar!(reject_empty_directories);
        scalar!(threads);
        macro_rules! optional_scalar {
            ($field:ident) => {
                if let Some(value) = &values.$field {
                    resolved.$field = Some(value.clone());
                    resolved.provenance.set(stringify!($field), source);
                }
            };
        }
        optional_scalar!(piece_length);
        optional_scalar!(source);
        optional_scalar!(comment);
        optional_scalar!(created_by);
        optional_scalar!(creation_date);
        optional_scalar!(name);
        let mut tracker_tiers = values
            .trackers
            .iter()
            .cloned()
            .map(|tracker| vec![tracker])
            .collect::<Vec<_>>();
        for alias in &values.tracker_aliases {
            tracker_tiers.push(vec![self.tracker(alias)?]);
        }
        for group in &values.tracker_groups {
            let group = self.file.tracker_groups.get(group).ok_or_else(|| {
                let suggestion =
                    suggestion(group, self.file.tracker_groups.keys().map(String::as_str))
                        .map_or_else(String::new, |candidate| {
                            format!("; did you mean {candidate:?}?")
                        });
                Error::metainfo_field(
                    "config.tracker_groups",
                    format!("unknown tracker group {group:?}{suggestion}"),
                )
            })?;
            let tier = group
                .trackers
                .iter()
                .map(|alias| self.tracker(alias))
                .collect::<Result<Vec<_>, _>>()?;
            tracker_tiers.push(tier);
        }
        if append_unique_tiers(&mut resolved.trackers, tracker_tiers) {
            resolved.provenance.set("trackers", source);
        }
        if append_unique(&mut resolved.web_seeds, values.web_seeds.iter().cloned()) {
            resolved.provenance.set("web_seeds", source);
        }
        if append_unique(&mut resolved.includes, values.includes.iter().cloned()) {
            resolved.provenance.set("includes", source);
        }
        if append_unique(&mut resolved.excludes, values.excludes.iter().cloned()) {
            resolved.provenance.set("excludes", source);
        }
        Ok(())
    }

    fn tracker(&self, alias: &str) -> Result<String, Error> {
        self.file
            .trackers
            .get(alias)
            .map(|tracker| tracker.url.clone())
            .ok_or_else(|| {
                let suggestion = suggestion(alias, self.file.trackers.keys().map(String::as_str))
                    .map_or_else(String::new, |candidate| {
                        format!("; did you mean {candidate:?}?")
                    });
                Error::metainfo_field(
                    "config.trackers",
                    format!("unknown tracker alias {alias:?}{suggestion}"),
                )
            })
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

pub(crate) fn run_command(cli: &Cli, arguments: &ConfigArgs) -> Result<(), Error> {
    let path = command_path(cli)?;
    match &arguments.command {
        ConfigCommand::Path => {
            stdout_line(path.display());
            Ok(())
        }
        ConfigCommand::Init { force } => init(&path, *force),
        ConfigCommand::Show {
            resolved,
            show_secrets,
            json,
        } => show(&path, *resolved, *show_secrets, *json),
        ConfigCommand::Check => check(&path),
        ConfigCommand::Explain(arguments) => match &arguments.command {
            ConfigExplainCommand::Create(create) => explain_create(&path, create),
        },
        ConfigCommand::Tracker(arguments) => tracker_command(&path, &arguments.command),
        ConfigCommand::Preset(arguments) => preset_command(&path, &arguments.command),
    }
}

fn command_path(cli: &Cli) -> Result<PathBuf, Error> {
    if cli.no_config {
        return Err(Error::metainfo_field(
            "config",
            "configuration commands cannot be used with --no-config",
        ));
    }
    cli.config
        .clone()
        .or_else(|| env::var_os("BTPC_CONFIG").map(PathBuf::from))
        .or_else(|| default_path(&Environment::current()))
        .ok_or_else(|| Error::metainfo_field("config", "cannot determine configuration path"))
}

fn init(path: &Path, force: bool) -> Result<(), Error> {
    if path.exists() && !force {
        return Err(Error::io(
            path,
            std::io::Error::new(std::io::ErrorKind::AlreadyExists, "already exists"),
        ));
    }
    let file = ConfigFile {
        version: CONFIG_VERSION,
        ..ConfigFile::default()
    };
    write_file(path, &file)?;
    stderr_line(format_args!("initialized {}", path.display()));
    Ok(())
}

fn show(path: &Path, resolved: bool, show_secrets: bool, json: bool) -> Result<(), Error> {
    let configuration = load_required(path)?;
    configuration.validate()?;
    if json {
        if show_secrets {
            return write_json(&configuration.file);
        }
        return write_json(&redacted_file(&configuration.file));
    }
    if !resolved && show_secrets {
        let text = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        stdout_text(text);
        return Ok(());
    }
    let file = if show_secrets {
        configuration.file
    } else {
        redacted_file(&configuration.file)
    };
    let text = toml::to_string_pretty(&file)
        .map_err(|error| Error::unsupported(format!("TOML encoding failed: {error}")))?;
    stdout_text(text);
    Ok(())
}

fn check(path: &Path) -> Result<(), Error> {
    let configuration = load_required(path)?;
    configuration.validate()?;
    check_permissions(path)?;
    stdout_line("valid");
    Ok(())
}

fn explain_create(path: &Path, arguments: &CreateArgs) -> Result<(), Error> {
    let configuration = load_optional(path)?;
    let resolved = configuration.resolve_create(arguments)?;
    validate_resolved(&resolved)?;
    for (field, value) in resolved_values(&resolved) {
        let source = resolved
            .provenance
            .values
            .get(field)
            .map_or("default", String::as_str);
        stdout_line(format_args!("{field}\t{value}\t{source}"));
    }
    Ok(())
}

fn tracker_command(path: &Path, command: &ConfigTrackerCommand) -> Result<(), Error> {
    match command {
        ConfigTrackerCommand::List { show_secrets, json } => {
            let configuration = load_optional(path)?;
            let values = configuration
                .file
                .trackers
                .iter()
                .map(|(name, tracker)| {
                    (
                        name.clone(),
                        if *show_secrets {
                            tracker.url.clone()
                        } else {
                            REDACTED_URL.to_owned()
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>();
            if *json {
                write_json(&values)
            } else {
                for (name, url) in values {
                    stdout_line(format_args!("{name}\t{url}"));
                }
                Ok(())
            }
        }
        ConfigTrackerCommand::Add { name, url } => mutate(path, |file| {
            file.trackers
                .insert(name.clone(), TrackerDefinition { url: url.clone() });
            Ok(())
        }),
        ConfigTrackerCommand::Remove { name } => mutate(path, |file| {
            if file.trackers.remove(name).is_none() {
                return Err(Error::metainfo_field(
                    "config.trackers",
                    format!("unknown tracker alias {name:?}"),
                ));
            }
            if file
                .tracker_groups
                .values()
                .any(|group| group.trackers.iter().any(|entry| entry == name))
                || file
                    .presets
                    .values()
                    .any(|preset| preset.tracker_aliases.iter().any(|entry| entry == name))
            {
                return Err(Error::metainfo_field(
                    "config.trackers",
                    format!("tracker alias {name:?} is still referenced"),
                ));
            }
            Ok(())
        }),
    }
}

fn preset_command(path: &Path, command: &ConfigPresetCommand) -> Result<(), Error> {
    match command {
        ConfigPresetCommand::List { json } => {
            let configuration = load_optional(path)?;
            let names = configuration
                .file
                .presets
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            if *json {
                write_json(&names)
            } else {
                for name in names {
                    stdout_line(name);
                }
                Ok(())
            }
        }
        ConfigPresetCommand::Show {
            name,
            show_secrets,
            json,
        } => {
            let configuration = load_optional(path)?;
            let preset = configuration.file.presets.get(name).ok_or_else(|| {
                let suggestion =
                    suggestion(name, configuration.file.presets.keys().map(String::as_str))
                        .map_or_else(String::new, |candidate| {
                            format!("; did you mean {candidate:?}?")
                        });
                Error::metainfo_field(
                    "config.presets",
                    format!("unknown preset {name:?}{suggestion}"),
                )
            })?;
            let redacted;
            let preset = if *show_secrets {
                preset
            } else {
                redacted = redacted_preset(preset);
                &redacted
            };
            if *json {
                write_json(preset)
            } else {
                let text = toml::to_string_pretty(preset).map_err(|error| {
                    Error::unsupported(format!("TOML encoding failed: {error}"))
                })?;
                stdout_text(text);
                Ok(())
            }
        }
        ConfigPresetCommand::Save(arguments) => mutate(path, |file| {
            file.presets
                .insert(arguments.name.clone(), preset_from_args(arguments));
            Ok(())
        }),
        ConfigPresetCommand::Remove { name } => mutate(path, |file| {
            if file.presets.remove(name).is_none() {
                return Err(Error::metainfo_field(
                    "config.presets",
                    format!("unknown preset {name:?}"),
                ));
            }
            if file
                .presets
                .values()
                .any(|preset| preset.extends.iter().any(|entry| entry == name))
            {
                return Err(Error::metainfo_field(
                    "config.presets",
                    format!("preset {name:?} is still extended"),
                ));
            }
            Ok(())
        }),
    }
}

fn preset_from_args(arguments: &PresetSaveArgs) -> Preset {
    Preset {
        extends: arguments.extends.clone(),
        mode: arguments.mode,
        piece_length: arguments.piece_length,
        trackers: arguments.trackers.clone(),
        tracker_aliases: arguments.tracker_aliases.clone(),
        tracker_groups: arguments.tracker_groups.clone(),
        web_seeds: arguments.web_seeds.clone(),
        private: arguments.private.then_some(true),
        source: arguments.source.clone(),
        comment: arguments.comment.clone(),
        created_by: arguments.created_by.clone(),
        creation_date: arguments.creation_date,
        name: arguments.name_override.clone(),
        exclude_hidden: arguments.exclude_hidden.then_some(true),
        symlinks: arguments.symlinks,
        special_files: arguments.special_files,
        exclude_empty_files: arguments.exclude_empty_files.then_some(true),
        reject_empty_directories: arguments.reject_empty_directories.then_some(true),
        includes: arguments.includes.clone(),
        excludes: arguments.excludes.clone(),
        threads: arguments.threads,
    }
}

fn mutate(
    path: &Path,
    operation: impl FnOnce(&mut ConfigFile) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut configuration = load_optional(path)?;
    operation(&mut configuration.file)?;
    configuration.validate()?;
    write_file(path, &configuration.file)
}

fn load_optional(path: &Path) -> Result<Configuration, Error> {
    if path.exists() {
        load_required(path)
    } else {
        Ok(Configuration {
            path: Some(path.to_path_buf()),
            file: ConfigFile {
                version: CONFIG_VERSION,
                ..ConfigFile::default()
            },
        })
    }
}

fn load_required(path: &Path) -> Result<Configuration, Error> {
    let contents = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let file = parse_file(&contents)?;
    Ok(Configuration {
        path: Some(path.to_path_buf()),
        file,
    })
}

fn parse_file(contents: &str) -> Result<ConfigFile, Error> {
    let file = toml::from_str::<ConfigFile>(contents).map_err(|error| {
        let location = error
            .span()
            .map_or_else(String::new, |span| format!(" at byte {}", span.start));
        Error::metainfo_field(
            "config",
            format!("invalid configuration TOML{location}: {}", error.message()),
        )
    })?;
    if file.version != CONFIG_VERSION {
        return Err(Error::metainfo_field(
            "config.version",
            format!(
                "unsupported configuration version {}; expected {CONFIG_VERSION}",
                file.version
            ),
        ));
    }
    Ok(file)
}

fn write_file(path: &Path, file: &ConfigFile) -> Result<(), Error> {
    write_file_with_hook(path, file, |_, _| Ok(()))
}

fn write_file_with_hook(
    path: &Path,
    file: &ConfigFile,
    before_persist: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), Error> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    let text = toml::to_string_pretty(file)
        .map_err(|error| Error::unsupported(format!("TOML encoding failed: {error}")))?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".btpc-config-")
        .tempfile_in(parent)
        .map_err(|source| Error::io(parent, source))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|source| Error::io(temporary.path(), source))?;
    }
    temporary
        .write_all(text.as_bytes())
        .map_err(|source| Error::io(temporary.path(), source))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|source| Error::io(temporary.path(), source))?;
    before_persist(temporary.path(), path).map_err(|source| Error::io(temporary.path(), source))?;
    temporary
        .persist(path)
        .map_err(|error| Error::io(path, error.error))?;
    Ok(())
}

fn redacted_file(file: &ConfigFile) -> ConfigFile {
    let mut redacted = file.clone();
    redacted.create.trackers.fill(REDACTED_URL.to_owned());
    redacted.create.web_seeds.fill(REDACTED_URL.to_owned());
    for tracker in redacted.trackers.values_mut() {
        REDACTED_URL.clone_into(&mut tracker.url);
    }
    for preset in redacted.presets.values_mut() {
        *preset = redacted_preset(preset);
    }
    redacted
}

fn redacted_preset(preset: &Preset) -> Preset {
    let mut redacted = preset.clone();
    redacted.trackers.fill(REDACTED_URL.to_owned());
    redacted.web_seeds.fill(REDACTED_URL.to_owned());
    redacted
}

fn check_permissions(path: &Path) -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = fs::metadata(path)
            .map_err(|source| Error::io(path, source))?
            .permissions()
            .mode()
            & 0o077;
        if mode != 0 {
            return Err(Error::metainfo_field(
                "config.permissions",
                "configuration permissions must be owner-only",
            ));
        }
    }
    Ok(())
}

fn validate_resolved(resolved: &ResolvedCreate) -> Result<(), Error> {
    if let Some(piece_length) = resolved.piece_length {
        btpc_core::create::validate_piece_length(
            piece_length,
            match resolved.mode {
                CliCreateMode::V1 => btpc_core::create::PieceLengthMode::V1,
                CliCreateMode::V2 | CliCreateMode::Hybrid => btpc_core::create::PieceLengthMode::V2,
            },
        )?;
    }
    Ok(())
}

fn resolved_values(resolved: &ResolvedCreate) -> Vec<(&'static str, String)> {
    vec![
        ("mode", format!("{:?}", resolved.mode).to_lowercase()),
        (
            "piece_length",
            resolved
                .piece_length
                .map_or_else(|| "automatic".to_owned(), |value| value.to_string()),
        ),
        ("trackers", resolved.trackers.len().to_string()),
        ("web_seeds", resolved.web_seeds.len().to_string()),
        ("private", resolved.private.to_string()),
        ("source", redacted_optional(resolved.source.as_ref())),
        ("comment", redacted_optional(resolved.comment.as_ref())),
        (
            "created_by",
            resolved.created_by.clone().unwrap_or_default(),
        ),
        (
            "creation_date",
            resolved
                .creation_date
                .map_or_else(String::new, |value| value.to_string()),
        ),
        ("name", resolved.name.clone().unwrap_or_default()),
        ("exclude_hidden", resolved.exclude_hidden.to_string()),
        (
            "symlinks",
            format!("{:?}", resolved.symlinks).to_lowercase(),
        ),
        (
            "special_files",
            format!("{:?}", resolved.special_files).to_lowercase(),
        ),
        (
            "exclude_empty_files",
            resolved.exclude_empty_files.to_string(),
        ),
        (
            "reject_empty_directories",
            resolved.reject_empty_directories.to_string(),
        ),
        ("includes", resolved.includes.join(",")),
        ("excludes", resolved.excludes.join(",")),
        ("threads", resolved.threads.to_string()),
    ]
}

fn redacted_optional(value: Option<&String>) -> String {
    value.map_or_else(String::new, |_| "<redacted>".to_owned())
}

#[allow(clippy::too_many_lines)]
fn apply_cli(resolved: &mut ResolvedCreate, arguments: &CreateArgs) {
    macro_rules! scalar {
        ($field:ident) => {
            if let Some(value) = &arguments.$field {
                resolved.$field = value.clone();
                resolved.provenance.set(stringify!($field), "cli");
            }
        };
    }
    scalar!(mode);
    scalar!(symlinks);
    scalar!(special_files);
    scalar!(threads);
    macro_rules! optional_scalar {
        ($field:ident) => {
            if let Some(value) = &arguments.$field {
                resolved.$field = Some(value.clone());
                resolved.provenance.set(stringify!($field), "cli");
            }
        };
    }
    optional_scalar!(piece_length);
    optional_scalar!(source);
    optional_scalar!(comment);
    optional_scalar!(created_by);
    if let Some(value) = arguments.creation_date {
        resolved.creation_date = match value {
            crate::command::CreationDateValue::None => None,
            crate::command::CreationDateValue::Timestamp(value) => Some(value),
        };
        resolved.provenance.set("creation_date", "cli");
    }
    optional_scalar!(name);
    if let Some(value) = arguments.target_pieces {
        resolved.target_pieces = Some(value);
    }
    if let Some(value) = arguments.max_piece_length {
        resolved.max_piece_length = Some(value);
    }
    if let Some(value) = &arguments.entropy {
        resolved.entropy = match value {
            crate::command::EntropyValue::None => None,
            crate::command::EntropyValue::Exact(bytes) => Some(bytes.clone()),
            crate::command::EntropyValue::Random => {
                let mut bytes = vec![0_u8; 16];
                if getrandom::fill(&mut bytes).is_ok() {
                    Some(bytes)
                } else {
                    None
                }
            }
        };
    }
    if arguments.clear_source {
        resolved.source = None;
    }
    if arguments.clear_comment {
        resolved.comment = None;
    }
    if arguments.clear_created_by {
        resolved.created_by = None;
        resolved.omit_created_by = true;
    } else if arguments.created_by.is_some() {
        resolved.omit_created_by = false;
    }
    if arguments.public {
        resolved.private = false;
        resolved.private_explicit = true;
    }
    for (flag, field, name) in [
        (arguments.private, &mut resolved.private, "private"),
        (
            arguments.exclude_hidden,
            &mut resolved.exclude_hidden,
            "exclude_hidden",
        ),
        (
            arguments.exclude_empty_files,
            &mut resolved.exclude_empty_files,
            "exclude_empty_files",
        ),
        (
            arguments.reject_empty_directories,
            &mut resolved.reject_empty_directories,
            "reject_empty_directories",
        ),
    ] {
        if flag {
            *field = true;
            if name == "private" {
                resolved.private_explicit = true;
            }
            resolved.provenance.set(name, "cli");
        }
    }
    if arguments.clear_trackers {
        resolved.trackers.clear();
        resolved.provenance.set("trackers", "cli-clear");
    }
    let mut tiers = arguments
        .trackers
        .iter()
        .cloned()
        .map(|tracker| vec![tracker])
        .collect::<Vec<_>>();
    if !arguments.tracker_tier.is_empty() {
        tiers.push(arguments.tracker_tier.clone());
    }
    if append_unique_tiers(&mut resolved.trackers, tiers) {
        resolved.provenance.set("trackers", "cli");
    }
    if arguments.clear_web_seeds {
        resolved.web_seeds.clear();
        resolved.provenance.set("web_seeds", "cli-clear");
    }
    if append_unique(&mut resolved.web_seeds, arguments.web_seeds.iter().cloned()) {
        resolved.provenance.set("web_seeds", "cli");
    }
    if arguments.clear_includes {
        resolved.includes.clear();
        resolved.provenance.set("includes", "cli-clear");
    }
    if append_unique(&mut resolved.includes, arguments.includes.iter().cloned()) {
        resolved.provenance.set("includes", "cli");
    }
    if arguments.clear_excludes {
        resolved.excludes.clear();
        resolved.provenance.set("excludes", "cli-clear");
    }
    if append_unique(&mut resolved.excludes, arguments.excludes.iter().cloned()) {
        resolved.provenance.set("excludes", "cli");
    }
}

fn append_unique(target: &mut Vec<String>, values: impl IntoIterator<Item = String>) -> bool {
    let mut seen = target.iter().cloned().collect::<BTreeSet<_>>();
    let original = target.len();
    for value in values {
        if seen.insert(value.clone()) {
            target.push(value);
        }
    }
    target.len() != original
}

fn append_unique_tiers(target: &mut Vec<Vec<String>>, values: Vec<Vec<String>>) -> bool {
    let mut seen = target.iter().cloned().collect::<BTreeSet<_>>();
    let original = target.len();
    for tier in values {
        if seen.insert(tier.clone()) {
            target.push(tier);
        }
    }
    target.len() != original
}

#[cfg(test)]
mod tests {
    use super::path_policy::{Environment, Platform, default_path_for};
    use super::{
        CONFIG_VERSION, ConfigFile, Configuration, CreateValues, Preset, write_file_with_hook,
    };
    use crate::command::{Cli, CliCreateMode};
    use clap::Parser as _;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn platform_default_path_uses_isolated_environment() {
        let environment = Environment {
            xdg_config_home: Some(PathBuf::from("/xdg")),
            home: Some(PathBuf::from("/home/test")),
            app_data: Some(PathBuf::from("C:/Users/test/AppData/Roaming")),
        };
        assert_eq!(
            default_path_for(Platform::Windows, &environment).unwrap(),
            PathBuf::from("C:/Users/test/AppData/Roaming/btpc/config.toml")
        );
        assert_eq!(
            default_path_for(Platform::Linux, &environment).unwrap(),
            PathBuf::from("/xdg/btpc/config.toml")
        );
        let without_xdg = Environment {
            xdg_config_home: None,
            ..environment
        };
        assert_eq!(
            default_path_for(Platform::Macos, &without_xdg).unwrap(),
            PathBuf::from("/home/test/Library/Application Support/btpc/config.toml")
        );
        assert_eq!(
            default_path_for(Platform::Linux, &without_xdg).unwrap(),
            PathBuf::from("/home/test/.config/btpc/config.toml")
        );
    }

    #[test]
    fn diamond_inheritance_deduplicates_lists_and_redacts_debug() {
        let mut presets = std::collections::BTreeMap::new();
        presets.insert(
            "base".to_owned(),
            Preset {
                web_seeds: vec!["https://secret.example/path?passkey=hidden".to_owned()],
                ..Preset::default()
            },
        );
        for name in ["left", "right"] {
            presets.insert(
                name.to_owned(),
                Preset {
                    extends: vec!["base".to_owned()],
                    ..Preset::default()
                },
            );
        }
        presets.insert(
            "top".to_owned(),
            Preset {
                extends: vec!["left".to_owned(), "right".to_owned()],
                mode: Some(CliCreateMode::V2),
                ..Preset::default()
            },
        );
        let configuration = Configuration {
            path: None,
            file: ConfigFile {
                version: CONFIG_VERSION,
                create: CreateValues::default(),
                presets,
                ..ConfigFile::default()
            },
        };
        let cli = Cli::try_parse_from(["btpc", "create", "payload", "--preset", "top"]).unwrap();
        let crate::command::Command::Create(arguments) = cli.command else {
            panic!("expected create command");
        };
        let mut resolved = configuration.resolve_create(&arguments).unwrap();
        resolved.source = Some("private source".to_owned());
        resolved.comment = Some("private comment".to_owned());
        resolved.entropy = Some(vec![1, 2, 3]);
        resolved.omit_created_by = true;
        assert_eq!(resolved.mode, CliCreateMode::V2);
        assert_eq!(resolved.web_seeds.len(), 1);
        let debug = format!("{configuration:?} {resolved:?}");
        assert!(!debug.contains("passkey=hidden"));
        assert!(!debug.contains("private source"));
        assert!(!debug.contains("private comment"));
        assert!(!debug.contains("[1, 2, 3]"));
        for field in [
            "tracker_count",
            "web_seed_count",
            "private_explicit",
            "created_by",
            "omit_created_by: true",
            "creation_date",
            "threads",
            "provenance",
        ] {
            assert!(
                debug.contains(field),
                "missing debug field {field}: {debug}"
            );
        }
    }

    #[test]
    fn failed_atomic_write_preserves_old_file_and_cleans_temporary_file() {
        let directory = tempfile::TempDir::new().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(&path, "old configuration\n").unwrap();
        let file = ConfigFile {
            version: CONFIG_VERSION,
            ..ConfigFile::default()
        };

        let result = write_file_with_hook(&path, &file, |_, _| {
            Err(std::io::Error::other("injected write failure"))
        });

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "old configuration\n");
        assert!(fs::read_dir(directory.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".btpc-config-")
        }));
    }
}
