# Technical Documentation

This is an introductory chapter into the technical layers of qaul.net.


## Introduction

Fundamentally, qaul.net is a highly distributed system.  It is not
accountable to a single set of rules across all devices that are part
of this system.  It is important to make the distinction between
"qaul.net", the application, "qaul network", the distributed network
of devices, and other components provided by the project that can be
used to write decentralised applications.

The primary component in this ecosystem is called `libqaul`.  It
provides an abstraction layer for a distributed network of devices,
user profiles that interact with each other, and the messages and data
that are exchanged.  The API of this library is called the "libqaul
service API" in other parts of the docs.  An application written to
use `libqaul` is called a "service".

A service provides more specific functionality.  Not all services are
user-facing.  For example, a service can provide an easy interface to
send rich-payload messages, that operates on a higher level than the
libqaul service API.  Other services can then depend on this
high-level API.

qaul.net (the app bundle) is primarily a GUI for a collection of
services, that all run on the same libqaul instance under the hood.

Unless otherwise stated, all code is written in Rust.


## Layers

The qaul.net stack is heavily abstracted into layers to keep logic
simple on each layer.  No layer should have to care about data that is
not meant to be consumed by it.

- **User Interfaces**: users interact with these clients
- **RPC broker**: allows remote clients to connect to the same daemon,
  without having to bundle their own libraries.
- **App services**: a set of services that provide user-facing
  functionality (text messaging, file sharing, ...)
- **Core services**: utility services that extend the `libqaul` API
- **libqaul**: primary user profile, and database handler
- **ratman**: decentralised packet router, responsible for dispatching
  and receiving messages.
- **Network Modules**: driver plugins for `ratman` that implement the
  actual wire-formats for networking
- **Platform Support**: os-specific utilities and tools


### Services & apps

A service (app) provides it's own API to clients, either via native
bindings, or via the common rpc-broker system.  Following is a list of
all the services that come bundled in qaul.net by default (more
details [here][services]).

- feed
- messages
- files
- voices
- radio

[services]: ./qaul.net/services.html

When connecting your own apps to a `libqaul` instance you can check
for the existence of other services, meaning that your application can
rely on extrenal functionality.  This way the binary bundles can be
kept small and focussed.


### RPC broker

To allow external applications to integrate with an existing qaul.net/
`libqaul` stack, the RPC message broker provides various interfaces to
integrate with.

- **[http/json]** - http server for `libqaul` and associated services
- **[socket-ipc]** - unix ipc socket interface with binary payloads
- **[android-ipc]** - a specific ipc implementation for Android

[http/json:api]: https://docs.qaul.net/http-api/
[socket-ipc]: ./libqaul/ipc/socket.html
[android-ipc]: ./libqaul/ipc/android.html


### libqaul

The primary state handler is called `libqaul`.  It handles database
transactions, local and remote user profiles, and connections to the
router.  Services can register themselves with a running instance for
authentication, to gain access to a per-service encrypted backing
storage.

While the main API is written (and accessible) in Rust, most services
will likely use the RPC broker system built on top of `libqaul`.


### ratman

A decentralised packet router, modelled partially on BATMAN-adv.  It
provides an API that takes messages to send to peers on the network,
and returns messages received from the network.  It handles network
announcements, network segmentation, message journaling, route
updates, and networked archive storage.  It's the main driver behind
qaul.net, and flexible enough to embed into various other use-cases.

Addresses on a ratman network are 32-byte ed25516 public keys, meaning
that all messages are automatically encrypted.  Additionally this
means that the valid address space isn't modelled on IP addresses, or
similar, and is nearly un-exhaustable.


### Network Modules

When sending messages over an internet overlay network, translation
between ratman IDs (provided by the `ratman-identity` crate) and the
various IP spaces needs to be performed.  This logic is implemented in
the network driver plugins (called "netmods").
