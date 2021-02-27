# Internals

The routing layer of qaul.net is made up of the `R.A.T.M.A.N.` routing
protocol.  It provides fully delay tolerant, distance vector based
cross-network mesh routing.  Check the graph below for a rough
overview of components.

![](/assets/dependencies.svg)

Currently `R.A.T.M.A.N.` has the following components.

## Core

Handles the routing table and frame routing workers. A thread listens
for `Frame`s to be sent out, while also polling for incoming frames
and sending them to a management thread.

The routing table can be modified behind an `Arc<Mutex<_>>`, allowing
other threads to inject user identity information into the worker
thread.

It also provides a `send` function which will queue a new set of
`Frame`s for sending. The `Core` does not handle `Message` slicing.

## Journal

This thread manages all `Frame`s that are being received by the
`Core`. `Frame`s that are addressed to a local user are buffered in a
cache until they can be re-merged into their respective
`Message`s. Foreign `Frame`s are being re-sent to the `Core` worker
thread, which will do an interface lookup and re-transmit them to the
net hop.

The local cache is is addressed by next-`Frame` signatures. The idea
here is that each `Frame` contains the signature of the next `Frame`
in the sequence (or `None`). This means that for each `Frame` it is
possible to tell if a following incoming `Frame` should be associated
to it, or if it might be wrong or even malicious.

## Slicer

This module doesn't provide it's own thread and instead only some
utilities for taking a `Message` and size hint, and turning that into
a set of `Frame` objects.

