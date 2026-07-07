mod creation;
mod editing;
mod errors;
mod inspection;
mod module;
mod progress;
mod verification;
mod views;

use btpc_core::create::{
    CreateMode, CreateOptions, Creator, DurabilityPolicy, HashThreads, OverwritePolicy, PieceLength,
};
use btpc_core::edit::MetainfoEditor;
use btpc_core::magnet::MagnetOptions;
use btpc_core::verify::{ExtraFilePolicy, MismatchMode, Verifier, VerifyOptions};
use btpc_core::{Metainfo, ParseLimits, ParseOptions};
use errors::to_python_error;
use progress::{PythonCancellationToken, PythonProgress};
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyAny, PyTuple};
use views::{
    NativeCreateResult, NativeMetainfo, NativePayloadMismatch, NativeVerificationReport,
    python_to_owned_value,
};

fn parse_options(
    max_total_input: Option<usize>,
    max_owned_allocation: Option<usize>,
    max_integer_digits: Option<usize>,
) -> ParseOptions {
    let defaults = ParseLimits::default();
    ParseOptions::new(
        ParseLimits::new(
            defaults.max_depth(),
            defaults.max_items(),
            defaults.max_byte_string_length(),
            max_total_input.unwrap_or(defaults.max_total_input()),
            max_owned_allocation.unwrap_or(defaults.max_owned_allocation()),
        )
        .with_max_integer_digits(max_integer_digits.unwrap_or(defaults.max_integer_digits())),
    )
}

#[pyfunction]
#[pyo3(signature = (data, max_total_input=None, max_owned_allocation=None, max_integer_digits=None))]
fn inspect_bytes(
    py: Python<'_>,
    data: &Bound<'_, PyAny>,
    max_total_input: Option<usize>,
    max_owned_allocation: Option<usize>,
    max_integer_digits: Option<usize>,
) -> PyResult<Py<NativeMetainfo>> {
    let options = parse_options(max_total_input, max_owned_allocation, max_integer_digits);
    let buffer = PyBuffer::<u8>::get(data).map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "data must support the contiguous byte buffer protocol",
        )
    })?;
    if !buffer.is_c_contiguous() {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "data must support the contiguous byte buffer protocol",
        ));
    }
    options
        .limits()
        .check_total_input(buffer.len_bytes())
        .map_err(|error| to_python_error(py, &error))?;
    let data = buffer.to_vec(py).map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err(
            "data must support the contiguous byte buffer protocol",
        )
    })?;
    let result = py.detach(move || Metainfo::from_owned_bytes_with_options(data, options));
    result
        .map_err(|error| to_python_error(py, &error))
        .and_then(|metainfo| Py::new(py, NativeMetainfo::new(metainfo)))
}

#[pyfunction]
#[pyo3(signature = (path, max_total_input=None, max_owned_allocation=None, max_integer_digits=None))]
fn inspect_path(
    py: Python<'_>,
    path: std::path::PathBuf,
    max_total_input: Option<usize>,
    max_owned_allocation: Option<usize>,
    max_integer_digits: Option<usize>,
) -> PyResult<Py<NativeMetainfo>> {
    let options = parse_options(max_total_input, max_owned_allocation, max_integer_digits);
    let result = py.detach(move || Metainfo::from_path_with_options(path, options));
    result
        .map_err(|error| to_python_error(py, &error))
        .and_then(|metainfo| Py::new(py, NativeMetainfo::new(metainfo)))
}

pub(crate) fn magnet_from_metainfo(
    metainfo: &Metainfo,
    display_name: bool,
    trackers: bool,
    web_seeds: bool,
) -> String {
    metainfo.magnet(
        &MagnetOptions::builder()
            .display_name(display_name)
            .trackers(trackers)
            .web_seeds(web_seeds)
            .build(),
    )
}

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
pub(crate) fn edit_metainfo(
    py: Python<'_>,
    metainfo: &Metainfo,
    trackers: Option<Vec<Vec<Vec<u8>>>>,
    web_seeds: Option<Vec<Vec<u8>>>,
    nodes: Option<Vec<(Vec<u8>, u16)>>,
    private: Option<bool>,
    set_private: bool,
    source: Option<Vec<u8>>,
    set_source: bool,
    comment: Option<Vec<u8>>,
    set_comment: bool,
    created_by: Option<Vec<u8>>,
    set_created_by: bool,
    creation_date: Option<i64>,
    set_creation_date: bool,
    raw_top_level: Vec<(Vec<u8>, Py<PyAny>)>,
    file_attributes: Vec<(Vec<Vec<u8>>, Vec<u8>)>,
) -> PyResult<Py<PyAny>> {
    let mut editor =
        MetainfoEditor::from_metainfo(metainfo).map_err(|error| to_python_error(py, &error))?;
    if let Some(trackers) = trackers {
        editor = editor.trackers(trackers);
    }
    if let Some(web_seeds) = web_seeds {
        editor = editor.web_seeds(web_seeds);
    }
    if let Some(nodes) = nodes {
        editor = editor.nodes(nodes);
    }
    if set_private {
        editor = editor.private(private);
    }
    if set_source {
        editor = editor.source(source);
    }
    if set_comment {
        editor = editor.comment(comment);
    }
    if set_created_by {
        editor = editor.created_by(created_by);
    }
    if set_creation_date {
        editor = editor.creation_date(creation_date);
    }
    for (key, value) in raw_top_level {
        let value = value.bind(py);
        let value = python_to_owned_value(py, value);
        editor = editor
            .raw_top_level(key, value?)
            .map_err(|error| to_python_error(py, &error))?;
    }
    for (path, attributes) in file_attributes {
        editor = editor
            .file_attributes(&path, attributes)
            .map_err(|error| to_python_error(py, &error))?;
    }
    let edited = editor
        .to_metainfo()
        .map_err(|error| to_python_error(py, &error))?;
    Py::new(py, NativeMetainfo::new(edited)).map(Py::into_any)
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (
    path,
    mode="v1",
    destination=None,
    overwrite=false,
    durable=false,
    piece_length=None,
    threads=None,
    trackers=Vec::new(),
    web_seeds=Vec::new(),
    nodes=Vec::new(),
    private=None,
    source=None,
    comment=None,
    created_by=None,
    omit_created_by=false,
    creation_date=None,
    progress=None,
    cancellation=None
))]
fn create_v1(
    py: Python<'_>,
    path: std::path::PathBuf,
    mode: &str,
    destination: Option<std::path::PathBuf>,
    overwrite: bool,
    durable: bool,
    piece_length: Option<u64>,
    threads: Option<usize>,
    trackers: Vec<Vec<Vec<u8>>>,
    web_seeds: Vec<Vec<u8>>,
    nodes: Vec<(Vec<u8>, u16)>,
    private: Option<bool>,
    source: Option<Vec<u8>>,
    comment: Option<Vec<u8>>,
    created_by: Option<Vec<u8>>,
    omit_created_by: bool,
    creation_date: Option<i64>,
    progress: Option<Py<PyAny>>,
    cancellation: Option<PyRef<'_, PythonCancellationToken>>,
) -> PyResult<Py<NativeCreateResult>> {
    let mode = parse_create_mode(mode)?;
    let mut builder = CreateOptions::builder()
        .mode(mode)
        .piece_length(piece_length.map_or(PieceLength::Automatic, PieceLength::Exact))
        .hash_threads(threads.map_or(HashThreads::Automatic, |threads| {
            if threads == 0 {
                HashThreads::Automatic
            } else {
                HashThreads::Exact(threads)
            }
        }))
        .trackers(trackers)
        .web_seeds(web_seeds)
        .nodes(nodes);
    if let Some(private) = private {
        builder = builder.private(private);
    }
    if let Some(source) = source {
        builder = builder.source(source);
    }
    if let Some(comment) = comment {
        builder = builder.comment(comment);
    }
    if omit_created_by {
        builder = builder.omit_created_by();
    } else if let Some(created_by) = created_by {
        builder = builder.created_by(created_by);
    }
    if let Some(creation_date) = creation_date {
        builder = builder.creation_date(creation_date);
    }
    let options = builder
        .build()
        .map_err(|error| to_python_error(py, &error))?;
    let cancellation = cancellation
        .map(|token| token.inner.clone())
        .unwrap_or_default();
    let progress = std::sync::Arc::new(PythonProgress::new(progress, cancellation.clone()));
    let operation_progress = std::sync::Arc::clone(&progress);
    let result = py.detach(move || {
        let creator = Creator::new(path)
            .options(options)
            .cancellation(cancellation);
        destination.map_or_else(
            || creator.create(operation_progress.as_ref()),
            |destination| {
                creator.create_to_path_with_durability(
                    destination,
                    if overwrite {
                        OverwritePolicy::Replace
                    } else {
                        OverwritePolicy::Deny
                    },
                    if durable {
                        DurabilityPolicy::FileAndDirectory
                    } else {
                        DurabilityPolicy::File
                    },
                    operation_progress.as_ref(),
                )
            },
        )
    });
    if let Some(error) = progress.take_error()? {
        return Err(error);
    }
    let result = result.map_err(|error| to_python_error(py, &error))?;
    Py::new(
        py,
        NativeCreateResult {
            inner: result,
            bytes: PyOnceLock::new(),
        },
    )
}

fn parse_create_mode(mode: &str) -> PyResult<CreateMode> {
    match mode {
        "v1" => Ok(CreateMode::V1),
        "v2" => Ok(CreateMode::V2),
        "hybrid" => Ok(CreateMode::Hybrid),
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "mode must be v1, v2, or hybrid",
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn verify_payload(
    py: Python<'_>,
    metainfo: Metainfo,
    payload: std::path::PathBuf,
    fail_fast: bool,
    extra_files: bool,
    progress: Option<Py<PyAny>>,
    cancellation: Option<PyRef<'_, PythonCancellationToken>>,
) -> PyResult<Py<NativeVerificationReport>> {
    let cancellation = cancellation
        .map(|token| token.inner.clone())
        .unwrap_or_default();
    let progress = std::sync::Arc::new(PythonProgress::new(progress, cancellation.clone()));
    let operation_progress = std::sync::Arc::clone(&progress);
    let result = py.detach(move || {
        let options = VerifyOptions::builder()
            .mismatch_mode(if fail_fast {
                MismatchMode::FailFast
            } else {
                MismatchMode::CollectAll
            })
            .extra_files(if extra_files {
                ExtraFilePolicy::Report
            } else {
                ExtraFilePolicy::Ignore
            })
            .build();
        Verifier::new(&metainfo, payload)
            .options(options)
            .cancellation(cancellation)
            .verify(operation_progress.as_ref())
    });
    if let Some(error) = progress.take_error()? {
        return Err(error);
    }
    let report = result.map_err(|error| to_python_error(py, &error))?;
    let mismatches = report
        .mismatches()
        .iter()
        .map(|mismatch| {
            Py::new(
                py,
                NativePayloadMismatch {
                    kind: mismatch_kind_name(mismatch.kind()),
                    path: mismatch.path().to_path_buf(),
                    piece: mismatch.piece(),
                },
            )
        })
        .collect::<PyResult<Vec<_>>>()?;
    let mismatches = PyTuple::new(py, mismatches)?.into_any().unbind();
    Py::new(py, NativeVerificationReport { mismatches })
}

const fn create_mode_name(mode: CreateMode) -> &'static str {
    match mode {
        CreateMode::V1 => "v1",
        CreateMode::V2 => "v2",
        CreateMode::Hybrid => "hybrid",
    }
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

#[pymodule(gil_used = true)]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module::register(module)
}
