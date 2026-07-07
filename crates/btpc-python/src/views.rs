use btpc_core::Metainfo;
use pyo3::class::basic::CompareOp;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyAny, PyBytes, PyDict, PyInt, PyList, PyTuple};

use crate::create_mode_name;
use crate::errors::to_python_error;
use crate::progress::PythonCancellationToken;

#[pyclass(name = "_NativeTorrentFile", module = "btpc._native", frozen)]
pub(crate) struct NativeTorrentFile {
    pub(crate) inner: btpc_core::TorrentFile,
    pub(crate) path: PyOnceLock<Py<PyAny>>,
    attributes: PyOnceLock<Py<PyAny>>,
    pieces_root: PyOnceLock<Py<PyAny>>,
}

#[pymethods]
impl NativeTorrentFile {
    #[getter]
    fn length(&self) -> u64 {
        self.inner.length()
    }

    #[getter]
    fn path(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.path
            .get_or_try_init(py, || {
                Ok(PyTuple::new(
                    py,
                    self.inner
                        .path_components()
                        .iter()
                        .map(|part| PyBytes::new(py, part)),
                )?
                .into_any()
                .unbind())
            })
            .map(|path| path.clone_ref(py))
    }

    #[getter]
    fn attributes(&self, py: Python<'_>) -> Py<PyAny> {
        self.attributes
            .get_or_init(py, || {
                PyBytes::new(py, self.inner.attributes())
                    .into_any()
                    .unbind()
            })
            .clone_ref(py)
    }

    #[getter]
    fn pieces_root(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner.pieces_root().map(|root| {
            self.pieces_root
                .get_or_init(py, || PyBytes::new(py, root).into_any().unbind())
                .clone_ref(py)
        })
    }

    #[getter]
    fn is_padding(&self) -> bool {
        self.inner.is_padding()
    }

    fn __repr__(&self) -> String {
        format!(
            "_NativeTorrentFile(length={}, path_components={})",
            self.inner.length(),
            self.inner.path_components().len()
        )
    }
}

#[pyclass(name = "_NativeValidationReport", module = "btpc._native", frozen)]
pub(crate) struct NativeValidationReport {
    warnings: Vec<String>,
    canonical: bool,
    canonical_offset: Option<usize>,
    canonical_message: Option<String>,
}

#[pymethods]
impl NativeValidationReport {
    #[getter]
    fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    #[getter]
    fn canonical(&self) -> bool {
        self.canonical
    }

    #[getter]
    fn canonical_offset(&self) -> Option<usize> {
        self.canonical_offset
    }

    #[getter]
    fn canonical_message(&self) -> Option<String> {
        self.canonical_message.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "_NativeValidationReport(canonical={}, warnings={})",
            self.canonical,
            self.warnings.len()
        )
    }
}

#[pyclass(name = "_NativeMetainfo", module = "btpc._native", frozen)]
pub(crate) struct NativeMetainfo {
    pub(crate) inner: Metainfo,
    files: PyOnceLock<Py<PyAny>>,
    trackers: PyOnceLock<Py<PyAny>>,
    web_seeds: PyOnceLock<Py<PyAny>>,
    nodes: PyOnceLock<Py<PyAny>>,
    unknown_fields: PyOnceLock<Py<PyAny>>,
    validation: PyOnceLock<Py<NativeValidationReport>>,
    original_bytes: PyOnceLock<Py<PyAny>>,
    canonical_bytes: PyOnceLock<Py<PyAny>>,
}

#[pyclass(name = "_NativeCreateResult", module = "btpc._native", frozen)]
pub(crate) struct NativeCreateResult {
    pub(crate) inner: btpc_core::create::CreateResult,
    pub(crate) bytes: PyOnceLock<Py<PyAny>>,
}

#[pyclass(name = "_NativePayloadMismatch", module = "btpc._native", frozen)]
pub(crate) struct NativePayloadMismatch {
    pub(crate) kind: &'static str,
    pub(crate) path: std::path::PathBuf,
    pub(crate) piece: Option<u64>,
}

#[pymethods]
impl NativePayloadMismatch {
    #[getter]
    fn kind(&self) -> &'static str {
        self.kind
    }

    #[getter]
    fn path(&self) -> &std::path::Path {
        &self.path
    }

    #[getter]
    fn piece(&self) -> Option<u64> {
        self.piece
    }

    fn __repr__(&self) -> String {
        format!(
            "_NativePayloadMismatch(kind='{}', path={}, piece={:?})",
            self.kind,
            self.path.display(),
            self.piece
        )
    }
}

#[pyclass(name = "_NativeVerificationReport", module = "btpc._native", frozen)]
pub(crate) struct NativeVerificationReport {
    pub(crate) mismatches: Py<PyAny>,
}

#[pymethods]
impl NativeVerificationReport {
    #[getter]
    fn mismatches(&self, py: Python<'_>) -> Py<PyAny> {
        self.mismatches.clone_ref(py)
    }

    #[getter]
    fn is_valid(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.mismatches.bind(py).len()? == 0)
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "_NativeVerificationReport(mismatches={})",
            self.mismatches.bind(py).len()?
        ))
    }
}

#[pymethods]
impl NativeCreateResult {
    #[getter]
    fn bytes(&self, py: Python<'_>) -> Py<PyAny> {
        self.bytes
            .get_or_init(py, || {
                PyBytes::new(py, self.inner.bytes()).into_any().unbind()
            })
            .clone_ref(py)
    }

    #[getter]
    fn mode(&self) -> &'static str {
        create_mode_name(self.inner.mode())
    }

    #[getter]
    fn info_hash_v1(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .info_hash_v1()
            .map(|hash| PyBytes::new(py, hash.as_bytes()).into_any().unbind())
    }

    #[getter]
    fn info_hash_v2(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .info_hash_v2()
            .map(|hash| PyBytes::new(py, hash.as_bytes()).into_any().unbind())
    }

    #[getter]
    fn file_count(&self) -> usize {
        self.inner.file_count()
    }

    #[getter]
    fn payload_bytes(&self) -> u64 {
        self.inner.payload_bytes()
    }

    #[getter]
    fn piece_count(&self) -> usize {
        self.inner.piece_count()
    }

    #[getter]
    fn piece_length(&self) -> u64 {
        self.inner.piece_length()
    }

    #[getter]
    fn piece_length_policy(&self) -> Option<&'static str> {
        self.inner.piece_length_policy()
    }

    #[getter]
    fn scan_ms(&self) -> f64 {
        self.inner.metrics().scan().as_secs_f64() * 1000.0
    }

    #[getter]
    fn hash_ms(&self) -> f64 {
        self.inner.metrics().hash().as_secs_f64() * 1000.0
    }

    #[getter]
    fn serialize_ms(&self) -> f64 {
        self.inner.metrics().serialize().as_secs_f64() * 1000.0
    }

    fn __repr__(&self) -> String {
        format!(
            "_NativeCreateResult(mode='{}', files={}, payload_bytes={})",
            create_mode_name(self.inner.mode()),
            self.inner.file_count(),
            self.inner.payload_bytes()
        )
    }
}

impl NativeMetainfo {
    pub(crate) fn new(inner: Metainfo) -> Self {
        Self {
            inner,
            files: PyOnceLock::new(),
            trackers: PyOnceLock::new(),
            web_seeds: PyOnceLock::new(),
            nodes: PyOnceLock::new(),
            unknown_fields: PyOnceLock::new(),
            validation: PyOnceLock::new(),
            original_bytes: PyOnceLock::new(),
            canonical_bytes: PyOnceLock::new(),
        }
    }
}

#[pymethods]
impl NativeMetainfo {
    fn __richcmp__(&self, other: &Self, operation: CompareOp) -> bool {
        match operation {
            CompareOp::Eq => self.inner.original_bytes() == other.inner.original_bytes(),
            CompareOp::Ne => self.inner.original_bytes() != other.inner.original_bytes(),
            _ => false,
        }
    }

    fn magnet(&self, display_name: bool, trackers: bool, web_seeds: bool) -> String {
        crate::magnet_from_metainfo(&self.inner, display_name, trackers, web_seeds)
    }

    #[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
    #[pyo3(signature = (
        trackers=None,
        web_seeds=None,
        nodes=None,
        private=None,
        set_private=false,
        source=None,
        set_source=false,
        comment=None,
        set_comment=false,
        created_by=None,
        set_created_by=false,
        creation_date=None,
        set_creation_date=false,
        raw_top_level=Vec::new(),
        file_attributes=Vec::new()
    ))]
    fn edit(
        &self,
        py: Python<'_>,
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
        crate::edit_metainfo(
            py,
            &self.inner,
            trackers,
            web_seeds,
            nodes,
            private,
            set_private,
            source,
            set_source,
            comment,
            set_comment,
            created_by,
            set_created_by,
            creation_date,
            set_creation_date,
            raw_top_level,
            file_attributes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (payload, fail_fast=false, extra_files=false, progress=None, cancellation=None))]
    fn verify(
        &self,
        py: Python<'_>,
        payload: std::path::PathBuf,
        fail_fast: bool,
        extra_files: bool,
        progress: Option<Py<PyAny>>,
        cancellation: Option<PyRef<'_, PythonCancellationToken>>,
    ) -> PyResult<Py<NativeVerificationReport>> {
        crate::verify_payload(
            py,
            self.inner.clone(),
            payload,
            fail_fast,
            extra_files,
            progress,
            cancellation,
        )
    }

    #[getter]
    fn mode(&self) -> &'static str {
        mode_name(self.inner.mode())
    }

    #[getter]
    fn name(&self, py: Python<'_>) -> Py<PyAny> {
        PyBytes::new(py, self.inner.name()).into_any().unbind()
    }

    #[getter]
    fn piece_length(&self) -> u64 {
        self.inner.piece_length()
    }

    #[getter]
    fn total_length(&self) -> u64 {
        self.inner.total_length()
    }

    #[getter]
    fn piece_count(&self) -> u64 {
        self.inner.piece_count()
    }

    #[getter]
    fn file_count(&self) -> usize {
        self.inner.files().len()
    }

    #[getter]
    fn files(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.files
            .get_or_try_init(py, || {
                let files = self
                    .inner
                    .files()
                    .iter()
                    .cloned()
                    .map(|inner| {
                        Py::new(
                            py,
                            NativeTorrentFile {
                                inner,
                                path: PyOnceLock::new(),
                                attributes: PyOnceLock::new(),
                                pieces_root: PyOnceLock::new(),
                            },
                        )
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(PyTuple::new(py, files)?.into_any().unbind())
            })
            .map(|files| files.clone_ref(py))
    }

    #[getter]
    fn trackers(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.trackers
            .get_or_try_init(py, || {
                let tiers = self
                    .inner
                    .trackers()
                    .iter()
                    .map(|tier| PyTuple::new(py, tier.iter().map(|value| PyBytes::new(py, value))))
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(PyTuple::new(py, tiers)?.into_any().unbind())
            })
            .map(|trackers| trackers.clone_ref(py))
    }

    #[getter]
    fn web_seeds(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.web_seeds
            .get_or_try_init(py, || {
                Ok(PyTuple::new(
                    py,
                    self.inner
                        .web_seeds()
                        .iter()
                        .map(|value| PyBytes::new(py, value)),
                )?
                .into_any()
                .unbind())
            })
            .map(|seeds| seeds.clone_ref(py))
    }

    #[getter]
    fn nodes(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.nodes
            .get_or_try_init(py, || {
                let nodes = self
                    .inner
                    .nodes()
                    .iter()
                    .map(|node| {
                        PyTuple::new(
                            py,
                            [
                                PyBytes::new(py, node.host()).into_any(),
                                node.port().into_pyobject(py)?.into_any(),
                            ],
                        )
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(PyTuple::new(py, nodes)?.into_any().unbind())
            })
            .map(|nodes| nodes.clone_ref(py))
    }

    #[getter]
    fn source(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .source()
            .map(|value| PyBytes::new(py, value).into_any().unbind())
    }

    #[getter]
    fn comment(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .comment()
            .map(|value| PyBytes::new(py, value).into_any().unbind())
    }

    #[getter]
    fn created_by(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .created_by()
            .map(|value| PyBytes::new(py, value).into_any().unbind())
    }

    #[getter]
    fn creation_date(&self) -> Option<i64> {
        self.inner.creation_date()
    }

    #[getter]
    fn private(&self) -> Option<bool> {
        self.inner.private()
    }

    #[getter]
    fn info_hash_v1(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .info_hash_v1()
            .map(|hash| PyBytes::new(py, hash.as_bytes()).into_any().unbind())
    }

    #[getter]
    fn info_hash_v2(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.inner
            .info_hash_v2()
            .map(|hash| PyBytes::new(py, hash.as_bytes()).into_any().unbind())
    }

    #[getter]
    fn original_bytes(&self, py: Python<'_>) -> Py<PyAny> {
        self.original_bytes
            .get_or_init(py, || {
                PyBytes::new(py, self.inner.original_bytes())
                    .into_any()
                    .unbind()
            })
            .clone_ref(py)
    }

    #[getter]
    fn canonical_bytes(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.canonical_bytes
            .get_or_try_init(py, || {
                let bytes = py
                    .detach(|| self.inner.to_bytes())
                    .map_err(|error| to_python_error(py, &error))?;
                Ok(PyBytes::new(py, &bytes).into_any().unbind())
            })
            .map(|bytes| bytes.clone_ref(py))
    }

    #[getter]
    fn canonical_cached(&self) -> bool {
        self.inner.canonical_bytes_cached()
    }

    #[getter]
    fn unknown_fields(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.unknown_fields
            .get_or_try_init(py, || {
                let fields = self
                    .inner
                    .unknown_fields()
                    .iter()
                    .map(|field| {
                        PyTuple::new(
                            py,
                            [
                                PyBytes::new(py, field.key()).into_any(),
                                owned_value_to_python(py, field.value())?.bind(py).clone(),
                                PyBytes::new(py, self.inner.unknown_field_bytes(field)).into_any(),
                                PyTuple::new(py, [field.span().start(), field.span().end()])?
                                    .into_any(),
                            ],
                        )
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                Ok(PyTuple::new(py, fields)?.into_any().unbind())
            })
            .map(|fields| fields.clone_ref(py))
    }

    #[getter]
    fn validation(&self, py: Python<'_>) -> PyResult<Py<NativeValidationReport>> {
        self.validation
            .get_or_try_init(py, || {
                let report = self.inner.validate();
                Py::new(
                    py,
                    NativeValidationReport {
                        warnings: report.warnings().to_vec(),
                        canonical: report.canonicality().is_canonical(),
                        canonical_offset: report.canonicality().offset(),
                        canonical_message: report.canonicality().message().map(str::to_owned),
                    },
                )
            })
            .map(|report| report.clone_ref(py))
    }

    fn __repr__(&self) -> String {
        format!(
            "_NativeMetainfo(mode='{}', name={:?}, files={})",
            mode_name(self.inner.mode()),
            String::from_utf8_lossy(self.inner.name()),
            self.inner.files().len()
        )
    }
}

pub(crate) fn owned_value_to_python(
    py: Python<'_>,
    value: &btpc_core::bencode::OwnedValue,
) -> PyResult<Py<PyAny>> {
    use btpc_core::bencode::OwnedValue;
    match value {
        OwnedValue::Integer(value) => Ok(value.into_pyobject(py)?.into_any().unbind()),
        OwnedValue::IntegerBytes(value) => {
            let builtins = py.import("builtins")?;
            Ok(builtins
                .getattr("int")?
                .call1((std::str::from_utf8(value).map_err(|_| {
                    pyo3::exceptions::PyValueError::new_err("invalid bencode integer")
                })?,))?
                .unbind())
        }
        OwnedValue::Bytes(value) => Ok(PyBytes::new(py, value).into_any().unbind()),
        OwnedValue::List(values) => {
            let values = values
                .iter()
                .map(|value| owned_value_to_python(py, value))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyList::new(py, values)?.into_any().unbind())
        }
        OwnedValue::Dictionary(entries) => {
            let dictionary = PyDict::new(py);
            for (key, value) in entries {
                dictionary.set_item(PyBytes::new(py, key), owned_value_to_python(py, value)?)?;
            }
            Ok(dictionary.into_any().unbind())
        }
    }
}

pub(crate) fn python_to_owned_value(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<btpc_core::bencode::OwnedValue> {
    use btpc_core::bencode::OwnedValue;
    if value.is_exact_instance_of::<PyInt>() {
        let encoded = value.str()?.to_str()?.as_bytes().to_vec();
        return OwnedValue::integer_bytes(encoded).map_err(|error| to_python_error(py, &error));
    }
    if let Ok(value) = value.cast::<PyBytes>() {
        return Ok(OwnedValue::bytes(value.as_bytes().to_vec()));
    }
    if let Ok(values) = value.cast::<PyList>() {
        return values
            .iter()
            .map(|value| python_to_owned_value(py, &value))
            .collect::<PyResult<Vec<_>>>()
            .map(OwnedValue::list);
    }
    if let Ok(entries) = value.cast::<PyDict>() {
        let mut converted = Vec::with_capacity(entries.len());
        for (key, value) in entries.iter() {
            let key = key.extract::<Vec<u8>>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err(
                    "raw extension dictionary keys must be bytes",
                )
            })?;
            converted.push((key, python_to_owned_value(py, &value)?));
        }
        return OwnedValue::dictionary(converted).map_err(|error| to_python_error(py, &error));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "raw extension values must be int, bytes, list, or dict",
    ))
}

pub(crate) const fn mode_name(mode: btpc_core::TorrentMode) -> &'static str {
    match mode {
        btpc_core::TorrentMode::V1 => "v1",
        btpc_core::TorrentMode::V2 => "v2",
        btpc_core::TorrentMode::Hybrid => "hybrid",
        _ => "unknown",
    }
}
