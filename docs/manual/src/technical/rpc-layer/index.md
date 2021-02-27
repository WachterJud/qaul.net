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
