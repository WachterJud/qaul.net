@0xf9778e7153e5e2bf;
# Base message definitions for the qrpc protocol


# A message is always from one component on the bus to another, meaning that it
# has an address and data section.  The broker doesn't need to understand the
# data types, so only the address is important
struct RpcMessage {
    id @0 :Text;
    addr @1 :Text;
    data @2 :Data;
}

