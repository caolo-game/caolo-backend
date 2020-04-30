// for protobuf
#[macro_use]
extern crate serde_derive;

use neon::prelude::*;

pub mod protos;

use cao_lang::compiler;

fn compile(mut cx: FunctionContext) -> JsResult<JsValue> {
    let arg = cx.argument::<JsValue>(0)?;

    let cu: compiler::CompilationUnit = neon_serde::from_value(&mut cx, arg)?;
    let res = compiler::compile(cu);
    let res = neon_serde::to_value(&mut cx, &res)?;

    Ok(res)
}

register_module!(mut cx, { cx.export_function("compile", compile) });
