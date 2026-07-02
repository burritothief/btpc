use crate::progress::PythonCancellationToken;
use crate::views::{
    NativeCreateResult, NativeMetainfo, NativePayloadMismatch, NativeTorrentFile,
    NativeValidationReport, NativeVerificationReport,
};
use pyo3::prelude::*;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    crate::errors::register(module.py(), module)?;
    module.add("__version__", btpc_core::version())?;
    module.add("__gil_required__", true)?;
    module.add("__subinterpreters_supported__", false)?;
    module.add_class::<NativeTorrentFile>()?;
    module.add_class::<NativeValidationReport>()?;
    module.add_class::<NativeMetainfo>()?;
    module.add_class::<NativeCreateResult>()?;
    module.add_class::<NativePayloadMismatch>()?;
    module.add_class::<NativeVerificationReport>()?;
    crate::inspection::register(module)?;
    crate::editing::register();
    crate::creation::register(module)?;
    crate::verification::register();
    module.add_class::<PythonCancellationToken>()
}
