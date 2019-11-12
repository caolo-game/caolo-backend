use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;

use cao_lang::prelude::{CompilationUnit, Compiler};

#[pyfunction]
/// Compile a graph
/// TODO: Pass the compilation unit as a dict
fn compile(py: Python, cu: String) -> PyResult<&PyDict> {
    let cu = serde_json::from_str::<CompilationUnit>(&cu).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!(
            "Can not deserialize the CompilationUnit {:?}",
            e
        ))
    })?;
    let _result = Compiler::compile(cu).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!("Failed to compile {:?}", e))
    })?;
    // TODO: return the compiled program
    Ok(PyDict::new(py))
}

#[pymodule]
fn caolo_web(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(compile))?;

    Ok(())
}
