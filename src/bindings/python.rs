use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyModule};

#[pyfunction]
#[pyo3(signature = (username, password, otp_callback=None, approval_callback=None))]
fn authenticate(
    py: Python<'_>,
    username: String,
    password: String,
    otp_callback: Option<Py<PyAny>>,
    approval_callback: Option<Py<PyAny>>,
) -> PyResult<String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;

    py.allow_threads(|| {
        runtime
            .block_on(crate::run_with_credentials(
                username,
                password,
                || {
                    let callback = otp_callback.as_ref().ok_or_else(|| {
                        "an otp_callback is required when phone OTP authentication is requested"
                            .to_string()
                    })?;
                    Python::with_gil(|py| {
                        callback
                            .call0(py)
                            .and_then(|value| value.extract::<String>(py))
                            .map_err(|error| format!("OTP callback failed: {error}"))
                    })
                },
                |number| {
                    let callback = approval_callback.as_ref().ok_or_else(|| {
                        "an approval_callback is required for Authenticator number matching"
                            .to_string()
                    })?;
                    Python::with_gil(|py| {
                        callback
                            .call1(py, (number,))
                            .map(|_| ())
                            .map_err(|error| format!("approval callback failed: {error}"))
                    })
                },
            ))
            .map_err(PyRuntimeError::new_err)
    })
}

#[pymodule]
fn uoe_ms_auth(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(authenticate, module)?)?;
    Ok(())
}