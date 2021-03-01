fn main() {
    let man = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    let protos_dir = format!("{}/../../protos", man);
    let protos = std::fs::read_dir(protos_dir.as_str())
        .expect("Failed to read protos directory")
        .filter(|path| {
            path.as_ref()
                .ok()
                .and_then(|p| {
                    Some(p.path().extension()? == "proto" && p.file_type().ok()?.is_file())
                })
                .unwrap_or(false)
        })
        .map(|p| p.as_ref().unwrap().path())
        .collect::<Vec<_>>();
    let protos = protos
        .iter()
        .map(|p| p.as_os_str().to_str().unwrap())
        .collect::<Vec<_>>();

    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(protos.as_slice())
        .include(protos_dir.as_str())
        .run()
        .expect("Failed to run protoc");
}
