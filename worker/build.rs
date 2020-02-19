use protoc_rust::Customize;
use std::fs::OpenOptions;
use std::io::Write;

const PROTOS: &'static [&'static str] = &[
    "../protos/scripts.proto",
    "../protos/input_messages.proto",
    "../protos/world.proto",
];

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: PROTOS,
        includes: &["../protos"],
        customize: Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc");

    let mut module_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("src/protos/mod.rs")
        .expect("mod.rs");

    for path in PROTOS
        .iter()
        .filter_map(|path| path.split("/").last().and_then(|x| x.split(".").next()))
    {
        writeln!(module_file, "pub mod {};", path).expect("write module file");
    }
}
