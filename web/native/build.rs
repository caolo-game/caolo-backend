use protoc_rust::Codegen;
use protoc_rust::Customize;
use std::fs::{read_dir, OpenOptions};
use std::io::Write;

const PROTOPATH: &str = "../../protos";

fn main() {
    neon_build::setup(); // must be called in build.rs

    let entries = read_dir(PROTOPATH);
    let protos = entries
        .unwrap()
        .into_iter()
        .filter_map(|p| p.ok())
        .filter(|p| {
            let meta = p.metadata().unwrap();
            meta.is_file()
                && p.path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains(".proto")
        })
        .collect::<Vec<_>>();

    let mut module_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("src/protos/mod.rs")
        .expect("mod.rs");

    for path in protos.iter() {
        let file_name = path.file_name();
        let module_name = file_name
            .to_str()
            .unwrap()
            .rsplitn(2, ".proto")
            .skip(1)
            .next()
            .unwrap();
        writeln!(module_file, "pub mod {};", module_name).expect("write module file");
    }

    Codegen::new()
        .include(PROTOPATH)
        .out_dir("src/protos")
        .inputs(protos.into_iter().map(|dirent| dirent.path()))
        .customize(Customize {
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc");
}
