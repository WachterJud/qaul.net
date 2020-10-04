# qaul.net rpc-layer

qaul.net (the application) is a set of services, that all communicate
with a shared backend.  This creates an extensible architecture

qaul.net (the application) is a set of services, all communicating
with a shared backend.  In the case of qaul-hubd, or the android app
for example, this is all done in the same binary.

However because qaul.net is an extensible architecture, it needs to be
possible for third-party developers to attach their services to an
already running system.  After all, there's littel use in creating
abstraction, if each third-party developer needs to host their own
router and database backend.

To accomplish this, the qaul.net rpc-layer contains several crates to
help out with this challenge.

## Overview

At the core lies the `qrpc-broker`.  It creates a server-client
architecture, with a shared set of [Capn Proto] wire formats to encode
APIs, function calls and concrete payload types.  To interface with
the broker as a client-lib, you need to use the `qrpc-sdk`, which
exposes utilities to create new wire format extensions.

The `qrpc-broker` backend will accept connections on various channels
(depending on the compiled configuration), which can be interacted
with via the client-libs for each channel.  Following is an example.


### Example: map service

You are an [OSM] enthusiast and you want to create an app that can
sync OSM data via qaul.net, and show dynamicly created points of
interest.  You also want people to have chats associated with each POI
that gets reported, but you don't want to have to handle group
encryption, and other problems.

[OSM]: https://openstreetmap.org

You write a service called `qaul-osm`, and use `qrpc-sdk` as a
dependency to create an API surface for your service.  You connect it
to the `qaul-hubd` rpc-broker running on a system, which gives you
access to using the `qaul-chat` service to implement the POI chat.


# qaul.net rpc-layer


Because qaul.net aims to be an extensible architecture, the core of
how services (apps) interact with each other is an RPC (remote
procedure call) layer.  This means that each service could be running
in a different process, and communicate with the core (the rpc-broker,
and libqaul instance) via sockets.

In actuality the main qaul.net services are all bundled into a single
binary (`qaul-hubd`) that communicate in memory to be more efficient.
But this doesn't have to be the case for others.

This page outlines some of the core concepts of the RPC layer, while
sub-pages go into more technical details, if you are interested in
working on a new feature for the RPC system.


## Registering a service

The rpc-broker keeps track of services that have registered themselves
on the system, and the capabilities they provide.
