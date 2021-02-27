# Overview of Crates

The qaul.net project largely exists in one mono-repo, meaning that all
relevant components are stored in the same git repository.  This makes
testing easier, but can make it daunting to look at because of the
sheer volume of code involved.

This list goes through the qaul.net application stack, starting at the
_bottom_ (driver/routing layer).

**Some of the crates in this list are WIP or don't yet fully exist, as
of 2020-02-08**

* **ratman/identity** is a crate that provides a simple identity
  abstraction for the rest of the ecosystem.  It has some utilities to
  create and serialise these identities, and is being used by almost
  all other libraries in the repository.  While you won't have to
  include this library manually, the re-exported `Identity` type is
  one you will see very often.
  
* **ratman/netmod** is a crate that defines the basic driver
  interaction layer between the router, and the network modules making
  connections available on different interfaces.  In common routing
  stacks this interface is provided by the kernel.  Since the ratman
  router works entirely in userspace, this provides a userspace
  abstraction (and scheduled) to send and receive basic datagrams.
  
* **ratman** is the main router crate, which exposes the `Router`
  type.  It also introduces the concept of messages, as a sequence of
  basic sealed data payloads.  There are utilities to interact with
  the basic protocol that makes a ratman network function, including
  announcements, key exchanges and data requests.  It's also fully
  delay tolerant, meaning that it can cache messages that couldn't be
  delivered yet.
  
* **libqaul** is the main application library for qaul.net.  It
  exposes the user concept, keystore, data store, and service API that
  allows the creation of decentralised, internet independent
  applications that communicate via a fully decentralised mesh
  network.  The main API is available for Rust programs, and other
  languages via on FFI wrapper.
  
* **libqaul/services** is a group of libraries that implement basic
  application functions on the qaul network.  These include a chat,
  filesharing, and more!
  
* **libqaul/rpc** is a library that wraps all function elements of the
  libqaul service API in serialisation structures.  By itself this
  crate doesn't do much, but can be used in combination with others to
  build applications that communicate with a qaul daemon via
  remote-procedure-calls.
  
* **libqaul/ws** is a websocket wrapper for the RPC interface, mostly
  used by the qaul.net web UI.
  
* **libqaul/ipc** is a unix socket wrapper for the RPC interface,
  that can be used to write applications on unix systems that talk to
  a qaul daemon.
  
* **libqaul/http** is an http, json RESTful wrapper for the RPC
  interface, that can be used from any language that has convenient
  http client libs, as well as used for debugging via tools like curL.
  
* **utils/ipc** is a client library for libqaul, that creates the
  sending end of the `libqaul-ipc` interface.
  
* **utils/ws.js** is a javascript wrapper for the websockets API,
  used mainly in the qaul.net web interface

* **clients/desktop-cli** is desktop CLI to run the qaul.net
  application, with an external service bundle
