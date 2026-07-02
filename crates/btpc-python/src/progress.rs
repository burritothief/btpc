use btpc_core::create::{CancellationToken, HashProgress, ProgressSink};
use pyo3::prelude::*;
use pyo3::types::PyAny;

#[pyclass(name = "_CancellationToken")]
#[derive(Default)]
pub(crate) struct PythonCancellationToken {
    pub(crate) inner: CancellationToken,
}

#[pymethods]
impl PythonCancellationToken {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn cancel(&self) {
        self.inner.cancel();
    }

    #[getter]
    fn cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }
}

pub(crate) struct PythonProgress {
    callback: Option<Py<PyAny>>,
    cancellation: CancellationToken,
    error: std::sync::Mutex<Option<PyErr>>,
    last_emit: std::sync::Mutex<Option<std::time::Instant>>,
}

impl PythonProgress {
    pub(crate) fn new(callback: Option<Py<PyAny>>, cancellation: CancellationToken) -> Self {
        Self {
            callback,
            cancellation,
            error: std::sync::Mutex::new(None),
            last_emit: std::sync::Mutex::new(None),
        }
    }

    pub(crate) fn take_error(&self) -> PyResult<Option<PyErr>> {
        self.error
            .lock()
            .map(|mut error| error.take())
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("progress lock poisoned"))
    }
}

impl ProgressSink for PythonProgress {
    fn on_progress(&self, progress: HashProgress) {
        let Some(callback) = &self.callback else {
            return;
        };
        if self.error.lock().is_ok_and(|error| error.is_some()) {
            return;
        }
        let now = std::time::Instant::now();
        let final_event = progress.bytes_hashed() == progress.total_bytes();
        let should_emit = self.last_emit.lock().is_ok_and(|mut last_emit| {
            if final_event
                || last_emit.is_none_or(|previous| {
                    now.duration_since(previous) >= std::time::Duration::from_millis(50)
                })
            {
                *last_emit = Some(now);
                true
            } else {
                false
            }
        });
        if !should_emit {
            return;
        }
        let callback_result = Python::attach(|py| {
            callback.call1(
                py,
                (
                    progress.bytes_hashed(),
                    progress.total_bytes(),
                    progress.pieces_hashed(),
                ),
            )
        });
        if let Err(error) = callback_result {
            self.cancellation.cancel();
            if let Ok(mut stored_error) = self.error.lock() {
                *stored_error = Some(error);
            }
        }
    }
}
