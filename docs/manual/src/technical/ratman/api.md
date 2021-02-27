# Ratman API

There's multiple scopes in the API that are responsible for doing
vastly different things.  This is due to how ratman can be used.
Within libqaul we use all three scopes throughout the application
lifecycle, but this isn't necessarily the case for all users of
Ratman.


## Interface setup

Because ratman does routing based on IDs and maps them to interfaces,
these links represent a network space reachable through them that will
allow a packet to be routed to it's final destination.  We call these
interface implementations "drivers" (more to that in the netmod docs).

Some are long-living, like the udp-netmod driver which grabs a local
network and relays traffic via multicast and unicast, others are
short-lived, such as the mem-netmod, which only represent a 1-to-1
link.  If the topology of the network were to change, the link gets
destroyed and cleaned up.

And that's what this API enables: a platform specific client which
wraps around various drivers can inject them into ratman, and teach it
about the network spaces that lie beyond in order to build up the
routing table.


## Routing

Using ratman to actually route traffic is pretty self explanatory: you
pass in a message (which is a ratman abstraction), in looks up where
to send it, slices it into frames according to the recomended packet
size for the link, and then dispatches it.


## Key and social callbacks

While the basic routing protocol is pretty simple there are ways to
suppliment it.  These callbacks are optional and ratman will happily
route packets without having access to this data (in fact you can
disable the features at compile time).  The idea is that having access
to a social graph for the routing can have positive effects on how
fast a packet can reach it's destination.  More docs to follow here,
mostly because a lot of it is based on pretty new network research.
