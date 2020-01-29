pub mod compiled_script_capnp {
    include!(concat!(env!("OUT_DIR"), "/compiled_script_capnp.rs"));
}
pub mod compiled_label_capnp {
    include!(concat!(env!("OUT_DIR"), "/compiled_label_capnp.rs"));
}

pub mod compiled_program_capnp {
    include!(concat!(env!("OUT_DIR"), "/compiled_program_capnp.rs"));
}
