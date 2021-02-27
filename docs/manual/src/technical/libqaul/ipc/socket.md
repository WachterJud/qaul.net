# Cap'n proto sockets

One way to inteact with libqaul is via the `libqaul-ipc` crate which
implements the same API as libqaul, while tunneling all calls through
a previously negotiated unix socket, using the cap'n proto IPC
protocol.

The same ffi C interface as for libqaul can be used, meaning that it's
possible to write a service that uses this high-performance IPC
channel from nearly any language.

More docs to follow, as this is still WIP!
