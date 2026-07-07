mod inspect_format;

use inspect_format::{display_url, format_creation_date, iec_size, inspect_magnet};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use btpc_core::bencode::{Value, ValueKind};
use btpc_core::create::{
    CreateMode, CreateOptions, Creator, DurabilityPolicy, EmptyDirectoryPolicy, EmptyFilePolicy,
    HashThreads, HiddenPolicy, ManifestOptions, OverwritePolicy, PieceLength, RootName,
    SpecialFilePolicy, SymlinkPolicy, automatic_piece_length, scan_manifest, write_atomic,
};
use btpc_core::edit::MetainfoEditor;
use btpc_core::magnet::MagnetOptions;
use btpc_core::metainfo::RawMetainfo;
use btpc_core::verify::{ExtraFilePolicy, MismatchMode, Verifier, VerifyOptions};
use btpc_core::{Error, ParseLimits, ParseOptions};
use serde::Deserialize;

use crate::command::{
    CliOutputFormat, CliSpecialFilePolicy, CliSymlinkPolicy, CreateArgs, CreatePrint, EditArgs,
    InspectArgs, InspectField, MagnetArgs, PathEncoding, ReadLimitArgs, ValidateArgs, VerifyArgs,
};
use crate::config::{Configuration, ResolvedCreate};
use crate::context::{ExecutionContext, OutputMode};
use crate::output::{
    CreateJson, DhtNodeJson, InspectJson, MetricsJson, ValidateJson, VerifyJson,
    VerifyMismatchJson, byte_string_json, display_bytes, filesystem_path_json, redacted_url_json,
    safe_path_display, stderr_line, stdout_line, stdout_path, stdout_text, write_json,
    write_json_pretty,
};
use crate::progress::CliProgress;
use crate::render::key_values;

pub(crate) fn magnet(arguments: &MagnetArgs) -> Result<(), Error> {
    let metainfo = btpc_core::Metainfo::from_path_with_options(
        &arguments.input,
        parse_options(&arguments.limits),
    )?;
    let options = MagnetOptions::builder()
        .display_name(!arguments.no_display_name)
        .trackers(!arguments.no_trackers)
        .web_seeds(!arguments.no_web_seeds)
        .build();
    stdout_line(metainfo.magnet(&options));
    Ok(())
}

// Spec: CLI-EDIT-001
#[allow(clippy::too_many_lines)]
pub(crate) fn edit(
    arguments: &EditArgs,
    context: &ExecutionContext,
    configuration: &Configuration,
) -> Result<(), Error> {
    let original = btpc_core::Metainfo::from_path(&arguments.input)?;
    let mut editor = MetainfoEditor::from_metainfo(&original)?;
    if arguments.clear_trackers
        || !arguments.trackers.is_empty()
        || !arguments.tracker_aliases.is_empty()
        || !arguments.tracker_groups.is_empty()
    {
        let tiers = configuration.resolve_tracker_tiers(
            &arguments.trackers,
            &arguments.tracker_aliases,
            &arguments.tracker_groups,
        )?;
        editor = editor.trackers(
            tiers
                .into_iter()
                .map(|tier| tier.into_iter().map(String::into_bytes).collect()),
        );
    }
    if arguments.clear_web_seeds || !arguments.web_seeds.is_empty() {
        editor = editor.web_seeds(
            arguments
                .web_seeds
                .iter()
                .map(|seed| seed.as_bytes().to_vec()),
        );
    }
    if arguments.clear_nodes || !arguments.nodes.is_empty() {
        editor = editor.nodes(arguments.nodes.clone());
    }
    if arguments.comment.is_some() || arguments.clear_comment {
        editor = editor.comment(
            arguments
                .comment
                .as_ref()
                .map(|value| value.as_bytes().to_vec()),
        );
    }
    if arguments.created_by.is_some() || arguments.clear_created_by {
        editor = editor.created_by(
            arguments
                .created_by
                .as_ref()
                .map(|value| value.as_bytes().to_vec()),
        );
    }
    if arguments.creation_date.is_some() || arguments.clear_creation_date {
        editor = editor.creation_date(arguments.creation_date);
    }
    if arguments.private || arguments.public || arguments.clear_private {
        editor = editor.private(if arguments.clear_private {
            None
        } else {
            Some(arguments.private)
        });
    }
    if arguments.source.is_some() || arguments.clear_source {
        editor = editor.source(
            arguments
                .source
                .as_ref()
                .map(|value| value.as_bytes().to_vec()),
        );
    }
    for (path, attributes) in &arguments.file_attributes {
        editor = editor.file_attributes(path, attributes.clone())?;
    }
    let edited = editor.to_metainfo()?;
    let destination = edit_destination(arguments);
    let v1_changed = original.info_hash_v1() != edited.info_hash_v1();
    let v2_changed = original.info_hash_v2() != edited.info_hash_v2();
    if !arguments.dry_run {
        let overwrite = if arguments.in_place || arguments.force {
            OverwritePolicy::Replace
        } else {
            OverwritePolicy::Deny
        };
        let durability = if arguments.durable {
            DurabilityPolicy::FileAndDirectory
        } else {
            DurabilityPolicy::File
        };
        write_atomic(&destination, edited.original_bytes(), overwrite, durability)?;
    }
    if arguments.json {
        write_json(&serde_json::json!({
            "schema": "btpc.edit.v2",
            "dry_run": arguments.dry_run,
            "output": filesystem_path_json(&destination),
            "output_display": safe_path_display(&destination),
            "info_hash_v1_changed": v1_changed,
            "info_hash_v2_changed": v2_changed,
            "info_hash_v1_before": original.info_hash_v1().map(|hash| hash.hex()),
            "info_hash_v1_after": edited.info_hash_v1().map(|hash| hash.hex()),
            "info_hash_v2_before": original.info_hash_v2().map(|hash| hash.hex()),
            "info_hash_v2_after": edited.info_hash_v2().map(|hash| hash.hex()),
        }))?;
    } else if context.human_output_enabled() {
        stdout_line(format_args!(
            "{}: {}",
            if arguments.dry_run {
                "would write"
            } else {
                "wrote"
            },
            safe_path_display(&destination)
        ));
        stdout_line(format_args!(
            "info hashes: v1 {} / v2 {}",
            changed_name(v1_changed),
            changed_name(v2_changed)
        ));
        if arguments.diff || context.verbosity() > 0 {
            stdout_line(format_args!(
                "v1: {} -> {}",
                optional_hash(original.info_hash_v1().map(|hash| hash.hex())),
                optional_hash(edited.info_hash_v1().map(|hash| hash.hex()))
            ));
            stdout_line(format_args!(
                "v2: {} -> {}",
                optional_hash(original.info_hash_v2().map(|hash| hash.hex())),
                optional_hash(edited.info_hash_v2().map(|hash| hash.hex()))
            ));
        }
    }
    Ok(())
}

fn edit_destination(arguments: &EditArgs) -> PathBuf {
    if arguments.in_place {
        return arguments.input.clone();
    }
    arguments.output.clone().unwrap_or_else(|| {
        arguments.input.with_file_name(filename_with_suffix(
            arguments.input.file_stem(),
            ".edited.torrent",
        ))
    })
}

const fn changed_name(changed: bool) -> &'static str {
    if changed { "changed" } else { "unchanged" }
}

fn optional_hash(hash: Option<String>) -> String {
    hash.unwrap_or_else(|| "n/a".to_owned())
}

pub(crate) fn verify(arguments: &VerifyArgs, context: &ExecutionContext) -> Result<(), Error> {
    let torrent = btpc_core::Metainfo::from_path_with_options(
        &arguments.torrent,
        parse_options(&arguments.limits),
    )?;
    let options = VerifyOptions::builder()
        .mismatch_mode(if arguments.fail_fast {
            MismatchMode::FailFast
        } else {
            MismatchMode::CollectAll
        })
        .extra_files(if arguments.extra_files {
            ExtraFilePolicy::Report
        } else {
            ExtraFilePolicy::Ignore
        })
        .build();
    let progress = CliProgress::new(context.progress_policy(), "verifying");
    let report = Verifier::new(&torrent, &arguments.payload)
        .options(options)
        .cancellation(context.cancellation())
        .verify(&progress)?;
    if context.output_mode() == OutputMode::Json {
        write_json(&VerifyJson {
            schema: "btpc.verify.v2",
            valid: report.is_valid(),
            mismatches: report
                .mismatches()
                .iter()
                .map(|mismatch| VerifyMismatchJson {
                    kind: mismatch_kind_name(mismatch.kind()),
                    path: filesystem_path_json(mismatch.path()),
                    deprecated_path_display: safe_path_display(mismatch.path()),
                    piece: mismatch.piece(),
                })
                .collect(),
        })?;
    } else if !context.human_output_enabled() {
    } else if report.is_valid() {
        stdout_line("valid");
    } else if context.human_output_enabled() {
        for mismatch in report.mismatches() {
            stdout_line(format_args!(
                "{}\t{}{}",
                mismatch_kind_name(mismatch.kind()),
                safe_path_display(mismatch.path()),
                mismatch
                    .piece()
                    .map_or_else(String::new, |piece| format!("\tpiece {piece}"))
            ));
        }
    }
    if report.is_valid() {
        Ok(())
    } else {
        Err(Error::verification_mismatch(
            None,
            "payload does not match metainfo",
        ))
    }
}

#[allow(clippy::too_many_lines)]
// Spec: CLI-INSPECT-002
pub(crate) fn inspect(arguments: &InspectArgs, context: &ExecutionContext) -> Result<(), Error> {
    let torrent = btpc_core::Metainfo::from_path_with_options(
        &arguments.input,
        parse_options(&arguments.limits),
    )?;
    let enhanced_human = context.output_mode() == OutputMode::Human
        && arguments.fields.is_empty()
        && !arguments.files
        && (arguments.tree || arguments.pretty || context.verbosity() > 0);
    let projection_requested = !arguments.fields.is_empty()
        || arguments.files
        || (!enhanced_human
            && (arguments.tree || arguments.offset != 0 || arguments.limit.is_some()))
        || matches!(
            arguments.format,
            Some(CliOutputFormat::Plain | CliOutputFormat::Tsv)
        );
    if !projection_requested
        && matches!(
            context.output_mode(),
            OutputMode::Json | OutputMode::JsonPretty
        )
    {
        let value = InspectJson {
            schema: "btpc.inspect.v1",
            mode: mode_name(torrent.mode()),
            name: byte_string_json(torrent.name()),
            total_bytes: torrent.total_length(),
            piece_length: torrent.piece_length(),
            piece_count: torrent.piece_count(),
            file_count: torrent.files().len(),
            info_hash_v1: torrent.info_hash_v1().map(|hash| hash.hex()),
            info_hash_v2: torrent.info_hash_v2().map(|hash| hash.hex()),
            trackers: torrent
                .trackers()
                .iter()
                .map(|tier| {
                    tier.iter()
                        .map(|tracker| redacted_url_json(tracker))
                        .collect()
                })
                .collect(),
            web_seeds: torrent
                .web_seeds()
                .iter()
                .map(|seed| redacted_url_json(seed))
                .collect(),
            nodes: torrent
                .nodes()
                .iter()
                .map(|node| DhtNodeJson {
                    host: byte_string_json(node.host()),
                    port: node.port(),
                })
                .collect(),
            source: torrent.source().map(byte_string_json),
            comment: torrent.comment().map(byte_string_json),
            created_by: torrent.created_by().map(byte_string_json),
            creation_date: torrent.creation_date(),
            private: torrent.private(),
            canonical: torrent.validate().canonicality().is_canonical(),
            warnings: torrent.validate().warnings().to_vec(),
        };
        if context.output_mode() == OutputMode::JsonPretty {
            write_json_pretty(&value)?;
        } else {
            write_json(&value)?;
        }
    } else if projection_requested {
        inspect_projection(arguments, context, &torrent)?;
    } else if context.output_mode() == OutputMode::Json {
        write_json(&InspectJson {
            schema: "btpc.inspect.v1",
            mode: mode_name(torrent.mode()),
            name: byte_string_json(torrent.name()),
            total_bytes: torrent.total_length(),
            piece_length: torrent.piece_length(),
            piece_count: torrent.piece_count(),
            file_count: torrent.files().len(),
            info_hash_v1: torrent.info_hash_v1().map(|hash| hash.hex()),
            info_hash_v2: torrent.info_hash_v2().map(|hash| hash.hex()),
            trackers: torrent
                .trackers()
                .iter()
                .map(|tier| {
                    tier.iter()
                        .map(|tracker| redacted_url_json(tracker))
                        .collect()
                })
                .collect(),
            web_seeds: torrent
                .web_seeds()
                .iter()
                .map(|seed| redacted_url_json(seed))
                .collect(),
            nodes: torrent
                .nodes()
                .iter()
                .map(|node| DhtNodeJson {
                    host: byte_string_json(node.host()),
                    port: node.port(),
                })
                .collect(),
            source: torrent.source().map(byte_string_json),
            comment: torrent.comment().map(byte_string_json),
            created_by: torrent.created_by().map(byte_string_json),
            creation_date: torrent.creation_date(),
            private: torrent.private(),
            canonical: torrent.validate().canonicality().is_canonical(),
            warnings: torrent.validate().warnings().to_vec(),
        })?;
    } else if context.human_output_enabled() {
        stdout_text(inspect_human(arguments, context, &torrent)?);
    }
    Ok(())
}

fn inspect_human(
    arguments: &InspectArgs,
    context: &ExecutionContext,
    torrent: &btpc_core::Metainfo,
) -> Result<String, Error> {
    let mut output = inspect_summary(torrent);
    if arguments.pretty {
        let validation = torrent.validate();
        let padding = torrent
            .files()
            .iter()
            .filter(|file| file.is_padding())
            .count();
        output.push_str("\nDetails:\n");
        writeln!(
            output,
            "  Source path: {}",
            safe_path_display(&arguments.input)
        )
        .expect("writing to String cannot fail");
        writeln!(
            output,
            "  Canonical: {}",
            if validation.canonicality().is_canonical() {
                "yes"
            } else {
                "no"
            }
        )
        .expect("writing to String cannot fail");
        writeln!(output, "  Total bytes: {}", torrent.total_length())
            .expect("writing to String cannot fail");
        writeln!(output, "  Piece bytes: {}", torrent.piece_length())
            .expect("writing to String cannot fail");
        writeln!(
            output,
            "  Payload files: {}",
            torrent.files().len() - padding
        )
        .expect("writing to String cannot fail");
        writeln!(output, "  Padding files: {padding}").expect("writing to String cannot fail");
        writeln!(output, "  Tracker tiers: {}", torrent.trackers().len())
            .expect("writing to String cannot fail");
        writeln!(output, "  Warnings: {}", validation.warnings().len())
            .expect("writing to String cannot fail");
    }
    if context.verbosity() > 0 {
        output.push_str("\nAdditional metadata:\n");
        append_optional_metadata(&mut output, torrent);
        for warning in torrent.validate().warning_details() {
            writeln!(
                output,
                "  Warning: {}{}{}",
                warning.message(),
                warning
                    .field()
                    .map_or(String::new(), |field| format!(" [field: {field}]")),
                warning
                    .offset()
                    .map_or(String::new(), |offset| format!(" [offset: {offset}]"))
            )
            .expect("writing to String cannot fail");
        }
        for field in torrent.unknown_fields() {
            writeln!(
                output,
                "  Unknown {}: {}",
                display_bytes(field.key()),
                bounded_owned_value(field.value())
            )
            .expect("writing to String cannot fail");
        }
        if torrent.comment().is_none()
            && torrent.created_by().is_none()
            && torrent.creation_date().is_none()
            && torrent.source().is_none()
            && torrent.validate().warnings().is_empty()
            && torrent.unknown_fields().is_empty()
        {
            output.push_str("  none\n");
        }
    }
    if arguments.tree || (context.verbosity() > 0 && torrent.files().len() > 1) {
        output.push('\n');
        output.push_str(&file_tree(arguments, torrent)?);
    }
    Ok(output)
}

fn append_optional_metadata(output: &mut String, torrent: &btpc_core::Metainfo) {
    for (label, value) in [
        ("Source", torrent.source()),
        ("Comment", torrent.comment()),
        ("Created by", torrent.created_by()),
    ] {
        if let Some(value) = value {
            writeln!(output, "  {label}: {}", display_bytes(value))
                .expect("writing to String cannot fail");
        }
    }
    if let Some(value) = torrent.creation_date() {
        writeln!(output, "  Creation date: {}", format_creation_date(value))
            .expect("writing to String cannot fail");
    }
    for node in torrent.nodes() {
        writeln!(
            output,
            "  DHT node: {}:{}",
            display_bytes(node.host()),
            node.port()
        )
        .expect("writing to String cannot fail");
    }
}

fn bounded_owned_value(value: &btpc_core::bencode::OwnedValue) -> String {
    use btpc_core::bencode::OwnedValue;
    let rendered = match value {
        OwnedValue::Integer(value) => value.to_string(),
        OwnedValue::IntegerBytes(value) | OwnedValue::Bytes(value) => display_bytes(value),
        OwnedValue::List(values) => format!("list({})", values.len()),
        OwnedValue::Dictionary(values) => format!("dictionary({})", values.len()),
    };
    if rendered.chars().count() > 120 {
        format!("{}...", rendered.chars().take(117).collect::<String>())
    } else {
        rendered
    }
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<Vec<u8>, TreeNode>,
    file: Option<(u64, bool)>,
}

fn file_tree(arguments: &InspectArgs, torrent: &btpc_core::Metainfo) -> Result<String, Error> {
    let end = arguments
        .limit
        .map_or(torrent.files().len(), |limit| {
            arguments.offset.saturating_add(limit)
        })
        .min(torrent.files().len());
    let start = arguments.offset.min(end);
    let selected = &torrent.files()[start..end];
    let mut root = TreeNode::default();
    for file in selected {
        let mut node = &mut root;
        for component in file.path_components() {
            node = node.children.entry(component.clone()).or_default();
        }
        node.file = Some((file.length(), file.is_padding()));
    }
    let mut output = String::from("File tree:\n");
    writeln!(output, "{}/", display_bytes(torrent.name())).expect("writing to String cannot fail");
    render_tree_nodes(&mut output, &root, "", arguments.path_encoding)?;
    let omitted = torrent.files().len() - selected.len();
    if omitted > 0 {
        writeln!(output, "... {omitted} file(s) omitted").expect("writing to String cannot fail");
    }
    Ok(output)
}

fn render_tree_nodes(
    output: &mut String,
    node: &TreeNode,
    prefix: &str,
    encoding: PathEncoding,
) -> Result<(), Error> {
    let count = node.children.len();
    for (index, (name, child)) in node.children.iter().enumerate() {
        let last = index + 1 == count;
        let connector = if last { "`-- " } else { "|-- " };
        let label = encode_path(std::slice::from_ref(name), encoding)?;
        output.push_str(prefix);
        output.push_str(connector);
        output.push_str(&label);
        if let Some((length, padding)) = child.file {
            write!(output, " ({})", iec_size(length)).expect("writing to String cannot fail");
            if padding {
                output.push_str(" [padding]");
            }
        } else {
            output.push('/');
        }
        output.push('\n');
        let mut next = prefix.to_owned();
        next.push_str(if last { "    " } else { "|   " });
        render_tree_nodes(output, child, &next, encoding)?;
    }
    Ok(())
}

// Spec: CLI-INSPECT-DISPLAY-001
fn inspect_summary(torrent: &btpc_core::Metainfo) -> String {
    let mut rows = vec![
        ("Name", display_bytes(torrent.name())),
        ("Mode", mode_name(torrent.mode()).to_owned()),
    ];
    if let Some(hash) = torrent.info_hash_v1() {
        rows.push(("Info hash v1", hash.hex()));
    }
    if let Some(hash) = torrent.info_hash_v2() {
        rows.push(("Info hash v2", hash.hex()));
    }
    rows.extend([
        ("Size", iec_size(torrent.total_length())),
        ("Piece length", iec_size(torrent.piece_length())),
        ("Pieces", torrent.piece_count().to_string()),
        ("Magnet", inspect_magnet(torrent)),
    ]);
    if torrent.files().len() > 1 {
        rows.push(("Files", torrent.files().len().to_string()));
    }
    if let Some(value) = torrent.private() {
        rows.push(("Private", if value { "yes" } else { "no" }.to_owned()));
    }
    if let Some(value) = torrent.source() {
        rows.push(("Source", display_bytes(value)));
    }
    if let Some(value) = torrent.comment() {
        rows.push(("Comment", display_bytes(value)));
    }
    if let Some(value) = torrent.created_by() {
        rows.push(("Created by", display_bytes(value)));
    }
    if let Some(value) = torrent.creation_date() {
        rows.push(("Creation date", format_creation_date(value)));
    }
    for node in torrent.nodes() {
        rows.push((
            "DHT node",
            format!("{}:{}", display_bytes(node.host()), node.port()),
        ));
    }
    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    let mut output = String::from("Torrent info:\n");
    for (label, value) in rows {
        writeln!(
            output,
            "  {label}:{} {value}",
            " ".repeat(width - label.len())
        )
        .expect("writing to String cannot fail");
    }
    if !torrent.trackers().is_empty() {
        output.push_str("  Trackers:\n");
        let multiple = torrent.trackers().len() > 1;
        for (index, tier) in torrent.trackers().iter().enumerate() {
            if multiple {
                writeln!(output, "    Tier {}:", index + 1).expect("writing to String cannot fail");
            }
            for tracker in tier {
                output.push_str("    ");
                if multiple {
                    output.push_str("  ");
                }
                output.push_str(&display_url(tracker));
                output.push('\n');
            }
        }
    }
    if !torrent.web_seeds().is_empty() {
        output.push_str("  Web seeds:\n");
        for seed in torrent.web_seeds() {
            output.push_str("    ");
            output.push_str(&display_url(seed));
            output.push('\n');
        }
    }
    output
}

fn inspect_projection(
    arguments: &InspectArgs,
    context: &ExecutionContext,
    torrent: &btpc_core::Metainfo,
) -> Result<(), Error> {
    let raw =
        RawMetainfo::from_bytes_with_options(torrent.original_bytes(), torrent.parse_options())?;
    let mut fields = arguments.fields.clone();
    if arguments.files || arguments.tree {
        fields.push(InspectField::Files);
    }
    if fields.is_empty() {
        fields = vec![
            InspectField::Mode,
            InspectField::Name,
            InspectField::TotalSize,
            InspectField::PieceLength,
            InspectField::PieceCount,
            InspectField::FileCount,
        ];
    }
    let values = fields
        .iter()
        .map(|field| {
            inspect_field_value(
                *field,
                torrent,
                &raw,
                arguments.path_encoding,
                arguments.tree,
                arguments.offset,
                arguments.limit,
            )
            .map(|value| (inspect_field_name(*field), value))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    match context.output_mode() {
        OutputMode::Json | OutputMode::JsonPretty => {
            let object = serde_json::json!({
                "schema": "btpc.inspect.selection.v1",
                "fields": values.iter().map(|(name, value)| serde_json::json!({"name": name, "value": value})).collect::<Vec<_>>(),
                "offset": arguments.offset,
                "limit": arguments.limit,
            });
            if context.output_mode() == OutputMode::JsonPretty {
                write_json_pretty(&object)
            } else {
                write_json(&object)
            }
        }
        OutputMode::Plain => {
            if let [(.., value)] = values.as_slice() {
                stdout_line(plain_value(value));
            } else {
                for (name, value) in values {
                    stdout_line(format_args!("{name}={}", plain_value(&value)));
                }
            }
            Ok(())
        }
        OutputMode::Tsv => {
            for (name, value) in values {
                stdout_line(format_args!(
                    "{}\t{}",
                    tsv_escape(name),
                    tsv_escape(&plain_value(&value))
                ));
            }
            Ok(())
        }
        OutputMode::Human => {
            let rows = values
                .into_iter()
                .map(|(name, value)| (name, plain_value(&value)))
                .collect::<Vec<_>>();
            stdout_text(key_values(&rows, context.pretty(), 80));
            Ok(())
        }
    }
}

fn inspect_field_value(
    field: InspectField,
    torrent: &btpc_core::Metainfo,
    raw: &RawMetainfo<'_>,
    encoding: PathEncoding,
    tree: bool,
    offset: usize,
    limit: Option<usize>,
) -> Result<serde_json::Value, Error> {
    Ok(match field {
        InspectField::Mode => serde_json::json!(mode_name(torrent.mode())),
        InspectField::Name => serde_json::json!(display_bytes(torrent.name())),
        InspectField::TotalSize => serde_json::json!(torrent.total_length()),
        InspectField::PieceLength => serde_json::json!(torrent.piece_length()),
        InspectField::PieceCount => serde_json::json!(torrent.piece_count()),
        InspectField::FileCount => serde_json::json!(torrent.files().len()),
        InspectField::HashV1 => torrent.info_hash_v1().map_or(serde_json::Value::Null, |value| serde_json::json!(value.hex())),
        InspectField::HashV2 => torrent.info_hash_v2().map_or(serde_json::Value::Null, |value| serde_json::json!(value.hex())),
        InspectField::Private => torrent.private().map_or(serde_json::Value::Null, serde_json::Value::Bool),
        InspectField::Trackers => serde_json::json!(torrent.trackers().iter().map(|tier| vec![crate::output::REDACTED_URL; tier.len()]).collect::<Vec<_>>()),
        InspectField::WebSeeds => serde_json::json!(vec![crate::output::REDACTED_URL; torrent.web_seeds().len()]),
        InspectField::Nodes => serde_json::json!(torrent.nodes().iter().map(|node| serde_json::json!({"host": display_bytes(node.host()), "port": node.port()})).collect::<Vec<_>>()),
        InspectField::Comment => torrent.comment().map_or(serde_json::Value::Null, |value| serde_json::json!(display_bytes(value))),
        InspectField::Creator => torrent.created_by().map_or(serde_json::Value::Null, |value| serde_json::json!(display_bytes(value))),
        InspectField::CreationDate => torrent.creation_date().map_or(serde_json::Value::Null, |value| serde_json::json!(value)),
        InspectField::Source => torrent.source().map_or(serde_json::Value::Null, |value| serde_json::json!(display_bytes(value))),
        InspectField::Canonicality => serde_json::json!(torrent.validate().canonicality().is_canonical()),
        InspectField::Warnings => serde_json::json!(torrent.validate().warnings()),
        InspectField::Files => {
            let files = torrent.files().iter().skip(offset).take(limit.unwrap_or(usize::MAX)).map(|file| {
                let path = encode_path(file.path_components(), encoding)?;
                Ok(serde_json::json!({"path": if tree { format!("{}{}", "  ".repeat(file.path_components().len().saturating_sub(1)), path) } else { path }, "length": file.length(), "attributes": display_bytes(file.attributes())}))
            }).collect::<Result<Vec<_>, Error>>()?;
            serde_json::Value::Array(files)
        }
        InspectField::UnknownFields => serde_json::json!(raw.unknown_fields().into_iter().map(|(key, value)| serde_json::json!({"key": display_bytes(key.bytes()), "value": value_text(value)})).collect::<Vec<_>>()),
    })
}

const fn inspect_field_name(field: InspectField) -> &'static str {
    match field {
        InspectField::Mode => "mode",
        InspectField::Name => "name",
        InspectField::TotalSize => "total-size",
        InspectField::PieceLength => "piece-length",
        InspectField::PieceCount => "piece-count",
        InspectField::FileCount => "file-count",
        InspectField::HashV1 => "hash-v1",
        InspectField::HashV2 => "hash-v2",
        InspectField::Private => "private",
        InspectField::Trackers => "trackers",
        InspectField::WebSeeds => "web-seeds",
        InspectField::Nodes => "nodes",
        InspectField::Comment => "comment",
        InspectField::Creator => "creator",
        InspectField::CreationDate => "creation-date",
        InspectField::Source => "source",
        InspectField::Canonicality => "canonicality",
        InspectField::Warnings => "warnings",
        InspectField::Files => "files",
        InspectField::UnknownFields => "unknown-fields",
    }
}

fn value_text(value: &Value<'_>) -> String {
    match value.kind() {
        ValueKind::Integer(value) => String::from_utf8_lossy(value.encoded()).into_owned(),
        ValueKind::Bytes(value) => display_bytes(value),
        ValueKind::List(values) => format!(
            "[{}]",
            values.iter().map(value_text).collect::<Vec<_>>().join(", ")
        ),
        ValueKind::Dictionary(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!(
                    "{}: {}",
                    display_bytes(key.bytes()),
                    value_text(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn encode_path(components: &[Vec<u8>], encoding: PathEncoding) -> Result<String, Error> {
    match encoding {
        PathEncoding::Utf8 => components
            .iter()
            .map(|component| {
                std::str::from_utf8(component)
                    .map(str::to_owned)
                    .map_err(|_| {
                        Error::metainfo_field(
                            "path",
                            "path is not valid UTF-8; use --path-encoding escaped or hex",
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|parts| parts.join("/")),
        PathEncoding::Escaped => Ok(components
            .iter()
            .map(|component| {
                component
                    .iter()
                    .map(|byte| {
                        if byte.is_ascii_graphic() && *byte != b'\\' {
                            char::from(*byte).to_string()
                        } else {
                            format!("\\x{byte:02x}")
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("/")),
        PathEncoding::Hex => Ok(components
            .iter()
            .map(|component| crate::output::encode_hex_public(component))
            .collect::<Vec<_>>()
            .join("/")),
    }
}

fn plain_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn tsv_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub(crate) fn validate(arguments: &ValidateArgs, context: &ExecutionContext) -> Result<(), Error> {
    let torrent = btpc_core::Metainfo::from_path_with_options(
        &arguments.input,
        parse_options(&arguments.limits),
    )?;
    let value = ValidateJson {
        schema: "btpc.validate.v1",
        valid: torrent.validate().is_valid()
            && (!arguments.canonical || torrent.validate().canonicality().is_canonical())
            && (!arguments.warnings_as_errors || torrent.validate().warnings().is_empty()),
        canonical: torrent.validate().canonicality().is_canonical(),
        warnings: torrent.validate().warnings().to_vec(),
    };
    if matches!(
        context.output_mode(),
        OutputMode::Json | OutputMode::JsonPretty
    ) {
        if context.output_mode() == OutputMode::JsonPretty {
            write_json_pretty(&value)?;
        } else {
            write_json(&value)?;
        }
    } else if context.human_output_enabled() {
        stdout_line(if value.valid { "valid" } else { "invalid" });
        for warning in torrent.validate().warnings() {
            stderr_line(format_args!("warning: {warning}"));
        }
    }
    if value.valid {
        Ok(())
    } else {
        Err(Error::metainfo_field(
            "validation",
            "requested validation policy failed",
        ))
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn create(
    arguments: &CreateArgs,
    context: &ExecutionContext,
    configuration: &Configuration,
) -> Result<(), Error> {
    let jobs = create_jobs(arguments)?;
    if arguments.jobs == 0 {
        return Err(Error::metainfo_field("jobs", "must be positive"));
    }
    let cancellation = context.cancellation();
    let signal_cancellation = cancellation.clone();
    ctrlc::set_handler(move || signal_cancellation.cancel())
        .map_err(|error| Error::unsupported(format!("cannot install Ctrl-C handler: {error}")))?;
    if jobs.len() > 1 && arguments.output.is_some() {
        return Err(Error::metainfo_field(
            "output",
            "--output is valid only for one input",
        ));
    }
    let mut destinations = BTreeSet::new();
    for job in &jobs {
        let destination = create_destination(job);
        if !destinations.insert(destination.clone()) {
            return Err(Error::metainfo_field(
                "output",
                format!(
                    "duplicate output destination {}",
                    safe_path_display(&destination)
                ),
            ));
        }
        if destination.exists() && !job.force {
            return Err(Error::io(
                destination,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "destination already exists",
                ),
            ));
        }
    }
    let mut first_error = None;
    for job in jobs {
        if let Err(error) = create_one(&job, context, configuration, cancellation.clone()) {
            if arguments.fail_fast {
                return Err(error);
            }
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

#[allow(clippy::too_many_lines)]
fn create_one(
    arguments: &CreateArgs,
    context: &ExecutionContext,
    configuration: &Configuration,
    cancellation: btpc_core::create::CancellationToken,
) -> Result<(), Error> {
    let input = arguments
        .inputs
        .first()
        .expect("create jobs always contain one input");
    let destination = arguments
        .output
        .clone()
        .unwrap_or_else(|| infer_output(input));
    let resolved = configuration.resolve_create(arguments)?;
    let manifest = manifest_options(&resolved)?;
    if arguments.dry_run {
        if destination.exists() && !arguments.force {
            return Err(Error::io(
                &destination,
                std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "destination already exists",
                ),
            ));
        }
        let scanned = scan_manifest(input, &manifest)?;
        let selected = selected_piece_length(&resolved, scanned.total_length());
        if context.human_output_enabled() {
            stdout_line(format_args!(
                "plan: {} files, {} bytes, piece length {}, output {}",
                scanned.entries().len(),
                scanned.total_length(),
                selected,
                safe_path_display(&destination)
            ));
        }
        return Ok(());
    }
    let options = create_options(&resolved, arguments.nodes.clone(), manifest)?;
    let creator = Creator::new(input)
        .options(options)
        .cancellation(cancellation);
    let overwrite = if arguments.force {
        OverwritePolicy::Replace
    } else {
        OverwritePolicy::Deny
    };
    let durability = if arguments.durable {
        DurabilityPolicy::FileAndDirectory
    } else {
        DurabilityPolicy::File
    };
    let progress = CliProgress::new(context.progress_policy(), "creating");
    let result =
        creator.create_to_path_with_durability(&destination, overwrite, durability, &progress)?;
    for value in &arguments.print {
        match value {
            CreatePrint::Path => stdout_path(&destination)?,
            CreatePrint::InfoHashV1 => stdout_line(
                result
                    .info_hash_v1()
                    .map_or_else(String::new, |hash| hash.hex()),
            ),
            CreatePrint::InfoHashV2 => stdout_line(
                result
                    .info_hash_v2()
                    .map_or_else(String::new, |hash| hash.hex()),
            ),
            CreatePrint::Magnet => {
                let metainfo = btpc_core::Metainfo::from_bytes(result.bytes())?;
                stdout_line(metainfo.magnet(&MagnetOptions::default()));
            }
        }
    }
    if context.output_mode() == OutputMode::Json {
        let metrics = result.metrics();
        write_json(&CreateJson {
            schema: "btpc.create.v2",
            mode: create_mode_name(result.mode()),
            output: filesystem_path_json(&destination),
            deprecated_output_display: safe_path_display(&destination),
            info_hash_v1: result.info_hash_v1().map(|hash| hash.hex()),
            info_hash_v2: result.info_hash_v2().map(|hash| hash.hex()),
            file_count: result.file_count(),
            payload_bytes: result.payload_bytes(),
            piece_count: result.piece_count(),
            piece_length: result.piece_length(),
            piece_length_policy: result.piece_length_policy(),
            metrics_ms: MetricsJson {
                scan: metrics.scan().as_millis(),
                hash: metrics.hash().as_millis(),
                serialize: metrics.serialize().as_millis(),
            },
        })?;
    } else if context.human_output_enabled() {
        let hashes = match (result.info_hash_v1(), result.info_hash_v2()) {
            (Some(v1), Some(v2)) => format!("SHA-1 {v1}, SHA-256 {v2}"),
            (Some(v1), None) => format!("SHA-1 {v1}"),
            (None, Some(v2)) => format!("SHA-256 {v2}"),
            (None, None) => unreachable!("every creation mode has an info hash"),
        };
        let prefix = if context.pretty() {
            "✓ created"
        } else {
            "created"
        };
        stderr_line(format_args!(
            "{prefix} {} ({} mode, {} files, {} bytes, {} pieces, {})",
            safe_path_display(&destination),
            create_mode_name(result.mode()),
            result.file_count(),
            result.payload_bytes(),
            result.piece_count(),
            hashes,
        ));
        if context.verbosity() > 0 {
            let metrics = result.metrics();
            stderr_line(format_args!(
                "timing: scan={}ms hash={}ms serialize={}ms",
                metrics.scan().as_millis(),
                metrics.hash().as_millis(),
                metrics.serialize().as_millis()
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchFile {
    version: u32,
    jobs: Vec<BatchJob>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchJob {
    input: PathBuf,
    output: Option<PathBuf>,
    #[serde(default)]
    presets: Vec<String>,
    mode: Option<crate::command::CliCreateMode>,
    piece_length: Option<u64>,
    threads: Option<usize>,
}

fn create_jobs(arguments: &CreateArgs) -> Result<Vec<CreateArgs>, Error> {
    // Spec: CLI-BATCH-001
    let mut jobs = if let Some(path) = &arguments.batch {
        let text = std::fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
        let batch = toml::from_str::<BatchFile>(&text).map_err(|error| {
            Error::metainfo_field("batch", format!("invalid batch TOML: {error}"))
        })?;
        if batch.version != 1 {
            return Err(Error::metainfo_field("batch.version", "expected version 1"));
        }
        batch
            .jobs
            .into_iter()
            .map(|job| {
                let mut resolved = arguments.clone();
                resolved.batch = None;
                resolved.inputs = vec![job.input];
                resolved.output = job.output;
                if resolved.presets.is_empty() {
                    resolved.presets = job.presets;
                }
                if resolved.mode.is_none() {
                    resolved.mode = job.mode;
                }
                if resolved.piece_length.is_none() {
                    resolved.piece_length = job.piece_length;
                }
                if resolved.threads.is_none() {
                    resolved.threads = job.threads;
                }
                resolved
            })
            .collect::<Vec<_>>()
    } else {
        arguments
            .inputs
            .iter()
            .cloned()
            .map(|input| {
                let mut job = arguments.clone();
                job.inputs = vec![input];
                job
            })
            .collect::<Vec<_>>()
    };
    if let Some(directory) = &arguments.output_dir {
        for job in &mut jobs {
            let input = job.inputs.first().expect("job input");
            job.output = Some(directory.join(filename_with_suffix(input.file_name(), ".torrent")));
        }
    }
    Ok(jobs)
}

fn create_destination(arguments: &CreateArgs) -> PathBuf {
    arguments
        .output
        .clone()
        .unwrap_or_else(|| infer_output(arguments.inputs.first().expect("create job input")))
}

fn parse_options(arguments: &ReadLimitArgs) -> ParseOptions {
    let defaults = ParseLimits::default();
    ParseOptions::new(
        ParseLimits::new(
            defaults.max_depth(),
            defaults.max_items(),
            defaults.max_byte_string_length(),
            arguments
                .max_input_bytes
                .unwrap_or(defaults.max_total_input()),
            arguments
                .max_owned_bytes
                .unwrap_or(defaults.max_owned_allocation()),
        )
        .with_max_integer_digits(
            arguments
                .max_integer_digits
                .unwrap_or(defaults.max_integer_digits()),
        ),
    )
}

const fn mismatch_kind_name(kind: btpc_core::verify::MismatchKind) -> &'static str {
    match kind {
        btpc_core::verify::MismatchKind::Missing => "missing",
        btpc_core::verify::MismatchKind::WrongSize => "wrong_size",
        btpc_core::verify::MismatchKind::Extra => "extra",
        btpc_core::verify::MismatchKind::UnsafePath => "unsafe_path",
        btpc_core::verify::MismatchKind::V1Hash => "v1_hash",
        btpc_core::verify::MismatchKind::V2Hash => "v2_hash",
    }
}

const fn mode_name(mode: btpc_core::TorrentMode) -> &'static str {
    match mode {
        btpc_core::TorrentMode::V1 => "v1",
        btpc_core::TorrentMode::V2 => "v2",
        btpc_core::TorrentMode::Hybrid => "hybrid",
        _ => "unknown",
    }
}

fn manifest_options(resolved: &ResolvedCreate) -> Result<ManifestOptions, Error> {
    let root_name = resolved.name.as_ref().map_or(RootName::Automatic, |name| {
        RootName::Override(name.as_bytes().to_vec())
    });
    ManifestOptions::builder()
        .hidden(if resolved.exclude_hidden {
            HiddenPolicy::Exclude
        } else {
            HiddenPolicy::Include
        })
        .symlinks(match resolved.symlinks {
            CliSymlinkPolicy::Reject => SymlinkPolicy::Reject,
            CliSymlinkPolicy::Skip => SymlinkPolicy::Skip,
            CliSymlinkPolicy::Follow => SymlinkPolicy::Follow,
        })
        .special_files(match resolved.special_files {
            CliSpecialFilePolicy::Reject => SpecialFilePolicy::Reject,
            CliSpecialFilePolicy::Skip => SpecialFilePolicy::Skip,
        })
        .empty_files(if resolved.exclude_empty_files {
            EmptyFilePolicy::Exclude
        } else {
            EmptyFilePolicy::Include
        })
        .empty_directories(if resolved.reject_empty_directories {
            EmptyDirectoryPolicy::Reject
        } else {
            EmptyDirectoryPolicy::Ignore
        })
        .include(resolved.includes.clone())
        .exclude(resolved.excludes.clone())
        .root_name(root_name)
        .build()
}

fn create_options(
    resolved: &ResolvedCreate,
    nodes: Vec<(Vec<u8>, u16)>,
    manifest: ManifestOptions,
) -> Result<CreateOptions, Error> {
    let piece_length = if let Some(value) = resolved.piece_length {
        PieceLength::Exact(value)
    } else if let Some(pieces) = resolved.target_pieces {
        PieceLength::Target {
            pieces,
            maximum: resolved.max_piece_length.unwrap_or(16 * 1024 * 1024),
        }
    } else {
        PieceLength::Automatic
    };
    let trackers = resolved
        .trackers
        .iter()
        .map(|tier| {
            tier.iter()
                .map(|tracker| tracker.as_bytes().to_vec())
                .collect()
        })
        .collect::<Vec<_>>();
    let mut builder = CreateOptions::builder()
        .manifest(manifest)
        .mode(resolved.mode.into())
        .piece_length(piece_length)
        .hash_threads(if resolved.threads == 0 {
            HashThreads::Automatic
        } else {
            HashThreads::Exact(resolved.threads)
        })
        .trackers(trackers)
        .web_seeds(
            resolved
                .web_seeds
                .iter()
                .map(|seed| seed.as_bytes().to_vec()),
        )
        .nodes(nodes);
    if resolved.private_explicit {
        builder = builder.private(resolved.private);
    }
    if let Some(source) = &resolved.source {
        builder = builder.source(source.as_bytes().to_vec());
    }
    if let Some(comment) = &resolved.comment {
        builder = builder.comment(comment.as_bytes().to_vec());
    }
    if let Some(created_by) = &resolved.created_by {
        builder = builder.created_by(created_by.as_bytes().to_vec());
    } else if resolved.omit_created_by {
        builder = builder.omit_created_by();
    }
    if let Some(creation_date) = resolved.creation_date {
        builder = builder.creation_date(creation_date);
    }
    if let Some(entropy) = &resolved.entropy {
        builder = builder.entropy(entropy.clone());
    }
    builder.build()
}

fn selected_piece_length(resolved: &ResolvedCreate, total_length: u64) -> u64 {
    if let Some(value) = resolved.piece_length {
        value
    } else if let Some(pieces) = resolved.target_pieces {
        total_length
            .div_ceil(pieces.max(1))
            .max(1024)
            .next_power_of_two()
            .min(resolved.max_piece_length.unwrap_or(16 * 1024 * 1024))
    } else {
        automatic_piece_length(total_length)
    }
}

const fn create_mode_name(mode: CreateMode) -> &'static str {
    match mode {
        CreateMode::V1 => "v1",
        CreateMode::V2 => "v2",
        CreateMode::Hybrid => "hybrid",
    }
}

fn infer_output(input: &Path) -> PathBuf {
    input.with_file_name(filename_with_suffix(input.file_name(), ".torrent"))
}

fn filename_with_suffix(name: Option<&std::ffi::OsStr>, suffix: &str) -> std::ffi::OsString {
    let mut output = name
        .unwrap_or_else(|| std::ffi::OsStr::new("output"))
        .to_os_string();
    output.push(suffix);
    output
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn inferred_names_preserve_colliding_non_utf8_bytes() {
        use super::filename_with_suffix;
        use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};

        let first =
            filename_with_suffix(Some(std::ffi::OsStr::from_bytes(b"name-\xff")), ".torrent");
        let second =
            filename_with_suffix(Some(std::ffi::OsStr::from_bytes(b"name-\xfe")), ".torrent");
        assert_eq!(first.into_vec(), b"name-\xff.torrent");
        assert_eq!(second.into_vec(), b"name-\xfe.torrent");
    }
}
