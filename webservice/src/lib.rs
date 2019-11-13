use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;

use cao_lang::prelude::{CompilationUnit, Compiler};

#[pyfunction]
/// Compile a graph
/// Pass the compilation unit as a JSON serialized string
fn compile(py: Python, cu: String) -> PyResult<&PyDict> {
    let cu = serde_json::from_str::<CompilationUnit>(&cu).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!(
            "Can not deserialize the CompilationUnit {:?}",
            e
        ))
    })?;
    let program = Compiler::compile(cu).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!("Failed to compile {:?}", e))
    })?;
    let result = PyDict::new(py);
    result.set_item("bytecode", program.bytecode).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!(
            "Failed to set the bytecode on the result {:?}",
            e
        ))
    })?;
    result.set_item("labels", program.labels).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!(
            "Failed to set the labels on the result {:?}",
            e
        ))
    })?;
    Ok(result)
}

#[pymodule]
fn caolo_web_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(compile))?;

    Ok(())
}
