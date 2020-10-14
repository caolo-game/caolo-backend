use serde::Serialize;
use cao_lang::prelude::SubProgramType;

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Function<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub ty: SubProgramType,
    pub input: Vec<&'a str>,
    pub output: Vec<&'a str>,
    pub constants: Vec<&'a str>,
}

impl<'a> Function<'a> {
    pub fn from_str_parts(
        name: &'a str,
        description: &'a str,
        ty: SubProgramType,
        input: &'a [&'a str],
        output: &'a [&'a str],
        constants: &'a [&'a str],
    ) -> Self {
        Self {
            name,
            description,
            ty,
            input: input.iter().map(|x| *x).collect(),
            output: output.iter().map(|x| *x).collect(),
            constants: constants.iter().map(|x| *x).collect(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Schema<'a> {
    pub functions: Vec<Function<'a>>,
}
