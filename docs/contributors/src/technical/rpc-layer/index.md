# qrpc: an application RPC layer

qaul.net (the application) is a set of services, that all communicate
with a shared backend.  This is done via the qrpc bus, goverened by a
central rpc broker.  Different parts of the ecosystem can communicate
with each other, without having to run in the same binary.

This way third-party developers can attach their services to an
already running system.  The qrpc-sdk is the primary way of
interacting with a qrpc bus, allowing you as a developer to query the
state of different components, check version compatibility, and more.

Each component on the qrpc bus is split into two parts: the api-crate,
and the impl-crate.  These are sometimes also called the server-lib,
and client-lib.  By including a thin API layer from each component in
your service you gain a fully typed API, that is communicating over
the qrpc bus under the hood.  For more details on how to use these
libraries, check out the [`qrpc-sdk`] crate documentation.

[`qrpc-sdk`]: https://docs.qaul.net/api/qrpc_sdk/index.html

Following are a few design documents that guide you through creating
your first qrpc service.
