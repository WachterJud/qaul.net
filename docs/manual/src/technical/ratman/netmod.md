# Netmod

This library is an abstraction that provides an interface and packet
specification that's used to communicate between Ratman and the
network drivers (also called "netmods").

The actual API is pretty simple: it provides functions to send and
poll for frames, provide a size hint (can be ignored), and link
strenth (used to break ties in the routing table).  Each call to send
and next will yield an extra "target" id which is also stored my
ratman to resolve one-to-many netmod links (like the udp one).

The API is also available from an ffi context where the I/O calls are
made blockingly. 


## Implementing a netmod in C++

The following docs are mostly notes and should be revised later.

Start by creating a type that can be used in C land. Give it all the
state it needs (here that means none).

```rust
#[repr(C)]
struct MyFfiDriver;
```

We also need some functions to inject state into, and constructing the
type:

```rust
extern "C" {
    fn ffi_driver_new() -> MyFfiDriver { ... }
    fn give(f: Frame) -> MyFfiDriver { ... }
    fn take() -> Fram { ... }
}
```

Make it implement the Endpoint and NativeEndpoint traits provided by
netmod (compiled with ffi support enabled).

```rust
#[async_trait]
impl Endpoint for MyFfiDriver {
    // ...
}
```

You can then make the Endpoint methods call the native methods where
appropriate, and the other way around.

In C++ land you can then include the `netmod.h` header, and use your
type that you construct via some additional out-of-trait constructor:

```C++
#include <ratman/netmod.h>

auto mod = ffi_driver_new();

mod.give(my_frame); // this is returned on 'next()' call

auto frame = mod.take(); // We can send this now
```
