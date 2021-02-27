# libqaul Internals

This section of the manual covers parts of libqaul not exposed
directly via the API.  It's primarily useful to learn to debug
behaviour, and for future contributors to get up to speed with the
internal structure.

Most of the components exposed via the API have an internal component
that acts as a strongly typed store (messages, users, contacts, files,
...).  Some more interesting components are those that are not exposed.


## Key Store

Similar to the user and message store, libqaul stores public keys that
it comes across, for later.  This is meant to opportunisticly fill the
store with keys that can be user later on to send encrypted messages
or verify signatures.

The store is shared between users, but never exposed to users so that
keys don't have to be fetched multiple times, but it's not
neccessarily possible for a user to figure out the social cycle of one
of their peers.

## Seeding

When a user is active on the network, this module sends out regular
"announcements" that are used to keep the routing tables up with
topology changes, as well as updates to local users.  Some seeds are
opportunistic, such as throwing out a public key, others are reactive,
driven by data requests that are picked up by the discovery module.

There are several strategies that can be chosen when seeding data into
the network, which should be given their own page soon.

## Discovery

The inverse of seeding to the network is the discovery module which
reacts to user seeds.  It will insert new users into the user store,
relay messages to the appropriate service, or dropping them if no
service handler was found.

## Persistence

The presistence module is implemented mostly by wrapping internal
libqaul types with [alexandria] storage callbacks, which is the
library which implements all of the persistence logic and at-rest
encryption.  It is developed as part of qaul.net, but pulled out of
the main tree to make it easier to use in other projects.

[alexandria]: https://git.open-communication.net/qaul/alexandria

## Routing

All calls to actually send data into a qaul network are implemented by
Ratman, the delay tolerant, distance vector based routing protocol
built specifically for qaul.net.  You can find documentation
specifically for it in later sections of this manual.

