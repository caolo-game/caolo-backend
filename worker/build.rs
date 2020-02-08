use protoc_rust::Customize;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["../protos/scripts.proto", "../protos/input_messages.proto"],
        includes: &["../protos"],
        customize: Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc");
}
