@0xf48ba5c2f4889b61;

# qrpc-sdk capabilities section
#
# Because qrpc doesn't use the capnproto RPC layer, each function call is
# mapped to a type.  The top-level of this abstraction is the `Capabilities`
# type, which is simply a tagged union (enum) to wrap around possible values.

using import "types.capnp".Service;

# Reply to any of the previous commands
struct SdkReply {
    union {
        hashId @0 :Text;
        success @1 :Bool;
    }
}


struct Capabilities {
    union {
        register @0 :Register;
        unregister @1 :Unregister;
        upgrade @2 :Upgrade;
    }
}

struct Register {
    service @0 :Service;
}

struct Unregister {
    hashId @0 :Text;
}

struct Upgrade {
    service @0 :Service;
    hashId @1 :Text;
}
