use capnpc::CompilerCommand as Cc;

fn main() {
    Cc::new()
        .file("schema/carrier.capnp")
        .run()
        .expect("Failed compiling schema/carrier.capnp!");
}
