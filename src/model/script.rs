use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Function<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub input: Vec<&'a str>,
    pub output: Vec<&'a str>,
    pub params: Vec<&'a str>,
}

impl<'a> Function<'a> {
    pub fn from_str_parts(
        name: &'a str,
        description: &'a str,
        input: &'a [&'a str],
        output: &'a [&'a str],
        params: &'a [&'a str],
    ) -> Self {
        Self {
            name,
            description,
            input: input.iter().map(|x| *x).collect(),
            output: output.iter().map(|x| *x).collect(),
            params: params.iter().map(|x| *x).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Schema<'a> {
    pub functions: Vec<Function<'a>>,
}
