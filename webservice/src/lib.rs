use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use std::collections::HashMap;

use cao_lang::compiler::description::get_instruction_descriptions;
use cao_lang::prelude::{compile as _compile, CompilationUnit};

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
    let program = _compile(cu)
        .map_err(|e| PyErr::new::<exceptions::ValueError, _>(format!("Failed to compile {}", e)))?;
    let result = PyDict::new(py);
    result.set_item("bytecode", program.bytecode).map_err(|e| {
        PyErr::new::<exceptions::ValueError, _>(format!(
            "Failed to set the bytecode on the result {:?}",
            e
        ))
    })?;
    result
        .set_item(
            "labels",
            program
                .labels
                .iter()
                .map(|(k, label)| (k, vec![label.block, label.myself]))
                .collect::<HashMap<_, _>>(),
        )
        .map_err(|e| {
            PyErr::new::<exceptions::ValueError, _>(format!(
                "Failed to set the labels on the result {:?}",
                e
            ))
        })?;
    Ok(result)
}

#[pyfunction]
fn get_basic_schema(py: Python) -> PyResult<&PyList> {
    let result = get_instruction_descriptions();
    let result = result
        .into_iter()
        .map(|desc| {
            let d = PyDict::new(py);
            d.set_item("name", desc.name).unwrap();
            d.set_item("description", desc.description).unwrap();
            d.set_item("output", desc.output).unwrap();
            d.set_item("input", desc.input).unwrap();
            d
        })
        .collect::<Vec<_>>();
    let lst = PyList::new(py, result);
    Ok(lst)
}

#[pymodule]
fn caolo_web_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(compile))?;
    m.add_wrapped(wrap_pyfunction!(get_basic_schema))?;

    Ok(())
}
