fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("../capnp")
        .file("../capnp/compiled_script.capnp")
        .file("../capnp/compiled_program.capnp")
        .file("../capnp/compiled_label.capnp")
        .run()
        .expect("schema compiler command")
}
