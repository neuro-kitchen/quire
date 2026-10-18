//! `quire_bindings`: the native module behind the `quire` Python SDK.

use pyo3::prelude::*;

#[pymodule]
fn quire_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
