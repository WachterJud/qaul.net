# Ratman

This is the documentation tree for the decentralised, delay-tolerant
vector routing protocol "Ratman".  It's basic design is influenced
heavily by protocols such as B.A.T.M.A.N., serval, and others.

In Ratman, all routing is done on IDs that are represented as the
blake2 hash of a user's public key.  It has a real-time and buffered
routing component: routing tables are built in userspace over a list
of interfaces and interface specific IDs (not the pubkey!), to
associate a pubkey with an interface route.  This relationship is
further explained in the netmod documentation.

If Ratman isn't able to find a route to a target, or based on various
parameters, it will cache a message in the local journal, periodically
checking if it can be delivered at a later point in time.

