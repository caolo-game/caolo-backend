use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use std::collections::HashMap;

use cao_lang::instruction::get_instruction_descriptions;
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
    result
        .set_item(
            "labels",
            program
                .labels
                .iter()
                .map(|(k, [x, y])| (k, vec![x, y]))
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
            d.set_item("desc", desc.desc).unwrap();
            d.set_item("output", desc.output).unwrap();
            d.set_item("inputs", desc.inputs).unwrap();
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
