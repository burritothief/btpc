use btpc_core::Error;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(btpc._native, NativeError, PyException);
create_exception!(btpc._native, NativeIoError, NativeError);
create_exception!(btpc._native, NativeBencodeSyntaxError, NativeError);
create_exception!(btpc._native, NativeBencodeCanonicalError, NativeError);
create_exception!(btpc._native, NativeMetainfoError, NativeError);
create_exception!(btpc._native, NativeResourceLimitError, NativeError);
create_exception!(btpc._native, NativeUnsupportedError, NativeError);
create_exception!(btpc._native, NativeVerificationError, NativeError);
create_exception!(btpc._native, NativeCancelledError, NativeError);

pub(crate) fn register(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("_NativeError", py.get_type::<NativeError>())?;
    module.add("_NativeIoError", py.get_type::<NativeIoError>())?;
    module.add(
        "_NativeBencodeSyntaxError",
        py.get_type::<NativeBencodeSyntaxError>(),
    )?;
    module.add(
        "_NativeBencodeCanonicalError",
        py.get_type::<NativeBencodeCanonicalError>(),
    )?;
    module.add("_NativeMetainfoError", py.get_type::<NativeMetainfoError>())?;
    module.add(
        "_NativeResourceLimitError",
        py.get_type::<NativeResourceLimitError>(),
    )?;
    module.add(
        "_NativeUnsupportedError",
        py.get_type::<NativeUnsupportedError>(),
    )?;
    module.add(
        "_NativeVerificationError",
        py.get_type::<NativeVerificationError>(),
    )?;
    module.add(
        "_NativeCancelledError",
        py.get_type::<NativeCancelledError>(),
    )
}

pub(crate) fn to_python_error(py: Python<'_>, error: &Error) -> PyErr {
    let value = match error {
        Error::Io { .. } => NativeIoError::new_err(error.to_string()),
        Error::BencodeSyntax { .. } => NativeBencodeSyntaxError::new_err(error.to_string()),
        Error::BencodeCanonical { .. } => NativeBencodeCanonicalError::new_err(error.to_string()),
        Error::Metainfo { .. } => NativeMetainfoError::new_err(error.to_string()),
        Error::ResourceLimit { .. } => NativeResourceLimitError::new_err(error.to_string()),
        Error::Unsupported { .. } => NativeUnsupportedError::new_err(error.to_string()),
        Error::Verification { .. } => NativeVerificationError::new_err(error.to_string()),
        Error::Cancelled => NativeCancelledError::new_err(error.to_string()),
        _ => NativeError::new_err(format!("unrecognized BTPC error: {error}")),
    };
    match attach_context(py, &value, error) {
        Ok(()) => value,
        Err(attachment_error) => attachment_error,
    }
}

fn attach_context(py: Python<'_>, value: &PyErr, error: &Error) -> PyResult<()> {
    let instance = value.value(py);
    instance.setattr("offset", error.offset())?;
    instance.setattr("field", error.field())?;
    instance.setattr("limit", error.limit())?;
    let (actual, maximum) = error
        .actual_and_maximum()
        .map_or((None, None), |(actual, maximum)| {
            (Some(actual), Some(maximum))
        });
    instance.setattr("actual", actual)?;
    instance.setattr("maximum", maximum)?;
    instance.setattr("path", error.path().map(filesystem_path_bytes))
}

#[cfg(unix)]
pub(crate) fn filesystem_path_bytes(path: &std::path::Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
pub(crate) fn filesystem_path_bytes(path: &std::path::Path) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}
