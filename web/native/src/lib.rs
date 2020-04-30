// for protobuf
#[macro_use]
extern crate serde_derive;

use log::{debug, warn};
use neon::prelude::*;
use std::cell::RefCell;

pub mod protos;

use cao_lang::{compiler, CompiledProgram};

pub struct CompilerTask {
    pub cu: RefCell<compiler::CompilationUnit>,
}

impl Task for CompilerTask {
    type Output = CompiledProgram;
    type Error = compiler::CompilationError;
    type JsEvent = JsValue;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let cu = RefCell::new(compiler::CompilationUnit::default());
        cu.swap(&self.cu);
        debug!("Compilation starting, {:?}", cu);
        compiler::compile(cu.into_inner())
            .map(|r| {
                debug!("Compilation succeeded {:?}", r);
                r
            })
            .map_err(|e| {
                warn!("Compilation failed {:?}", e);
                e
            })
    }

    fn complete(
        self,
        mut cx: TaskContext,
        res: Result<Self::Output, Self::Error>,
    ) -> JsResult<JsValue> {
        let res = neon_serde::to_value(&mut cx, &res)?;
        Ok(res)
    }
}

pub fn compile(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let arg = cx.argument::<JsValue>(0)?;
    let callback = cx.argument::<JsFunction>(1)?;

    let cu: compiler::CompilationUnit = neon_serde::from_value(&mut cx, arg)?;
    let cu = RefCell::new(cu);
    let compiler = CompilerTask { cu };
    compiler.schedule(callback);

    Ok(cx.undefined())
}

pub fn init(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    #[cfg(feature = "dotenv")]
    dep_dotenv::dotenv().unwrap_or_default();

    env_logger::init();
    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("compile", compile)?;
    cx.export_function("init", init)
});
