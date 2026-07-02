use pyo3::prelude::*;

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(crate::inspect_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(crate::inspect_path, module)?)
}
