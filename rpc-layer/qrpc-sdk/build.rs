use capnpc::CompilerCommand as Cc;

fn main() {
    Cc::new()
        .file("schema/base.capnp") // base wire wrapper
        .file("schema/types.capnp") // sdk-data types
        .file("schema/cap.capnp") // rpc-api types
        .run()
        .expect("Failed compiling schema/carrier.capnp!");
}
