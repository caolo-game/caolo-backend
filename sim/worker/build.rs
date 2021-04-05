fn main() {
    let man = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    let protos_dir = format!("{}/../../protos", man);
    println!("cargo:rerun-if-changed={}", protos_dir);

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

    tonic_build::configure()
        .compile(protos.as_slice(), &[protos_dir.as_str()])
        .expect("Failed to run protoc");
}
