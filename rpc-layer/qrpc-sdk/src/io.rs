//! An I/O abstraction module for the qrpc system
//!
//! The qrpc system heavily builds on capnproto as an exchange and RPC
//! format.  Unfortunately the capnproto-rs interface is pretty shit
//! (this is rude, I know but it's just not great...).  Having to
//! interact with it to write services for qaul.net might be a dealbreaker.
//!
//! And so... this module tries to abstract as much of the low-level
//! ugliness away.  Instead, you pass it a buffer with a message, and
//! it parses it for you, with some simple type constraits that are
//! easy to enforce in your application.  Additionally, it exposes
//! some more convenient builders as well (although the builder APIs
//! in the original crate are quite good).

use capnp::{
    message::{Reader, ReaderOptions},
    serialize::OwnedSegments,
    serialize_packed as ser,
    traits::FromPointerReader,
};
use std::marker::PhantomData;

/// A result-wrapper for capnproto related failures
pub type Result<T> = capnp::Result<T>;

/// A utility type to read capnproto message types
pub struct MsgReader<'s, T: FromPointerReader<'s>> {
    r: Reader<OwnedSegments>,
    _t: &'s PhantomData<T>,
}

impl<'s, T: FromPointerReader<'s>> MsgReader<'s, T> {
    /// Parse a message buffer into a set of owned segments
    pub fn new(buf: Vec<u8>) -> Result<Self> {
        ser::read_message(buf.as_slice(), ReaderOptions::new()).map(|r| Self {
            r,
            _t: &PhantomData,
        })
    }

    /// Get the root object from this reader, if it exists
    ///
    /// This function returns a reference to the inner reader for you.
    /// Because the way this trait is implemented, the parent can't go
    /// out of scope.
    ///
    /// To get access to the fields of a type, you need to type-cast
    /// it as a `T::Reader`, so to read a `service` type (such as the
    /// one provided by this sdk crate), you would cast it as
    /// `service::Reader`.
    ///
    /// ```
    /// # use qrpc_sdk::io::Result;
    /// use qrpc_sdk::{io::MsgReader, types::service};
    ///
    /// # fn run_code() -> Result<()> {
    /// # let buf = vec![];
    /// let msg = MsgReader::new(buf)?;
    /// let r: service::Reader = msg.get_root()?;
    /// println!("DESC: {}", r.get_description()?);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Some types will be capability sets, encoded in an unnamed
    /// union.  Because this is a very common pattern, here is an
    /// example usage of how to implement matching for the types
    /// defined in this crate.
    ///
    /// ```
    /// use qrpc_sdk::{io::MsgReader, rpc::capabilities::{Reader, Which}};
    /// # fn foo() -> Result<(), Box<std::error::Error>> {
    /// # let reader = qrpc_sdk::builders::parse_rpc_msg(vec![]).unwrap();
    /// 
    /// // Get the `reader` by calling `builders::parse_rpc_msg(...)`
    /// let r: Reader = reader.get_root().unwrap();
    /// match r.which() {
    ///     Ok(Which::Register(Ok(reg))) => handle_register(reg),
    ///     Ok(Which::Unregister(Ok(unreg))) => handle_unregister(unreg),
    ///     Ok(Which::Upgrade(Ok(upgr))) => handle_upgrade(uphr),
    ///     _ => eprintln!("Invalid variant/ decode!"),
    /// }
    /// # Ok(())
    /// # }
    ///
    /// use qrpc_sdk::rpc::{register, unregister, upgrade};
    ///
    /// fn handle_register(_: register::Reader) {}
    /// fn handle_unregister(_: unregister::Reader) {}
    /// fn handle_upgrade(_: upgrade::Reader) {}
    /// ```
    ///
    /// The above code can be found in the [qrpc-broker] crate.  Your
    /// own service code will differ, but this should give you a good
    /// idea how to start!
    ///
    /// [qrpc-broker]: https://docs.qaul.net/api/qrpc_broker/index.html
    pub fn get_root(&'s self) -> Result<T> {
        self.r.get_root()
    }
}
