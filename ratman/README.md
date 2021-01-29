# Ratman

A modular userspace frame router, implementing distance vector
routing, and delay tolerance.  Handles topology updates and user
discovery via flood heartbeats, and provides a non-namespaced view
into a network via ed25519 keyed user IDs.

One of the core principles of Ratman is to make network roaming
easier, building a general abstraction over a network, leaving it up
to drivers to interface with implementation specifics.

As such, the Ratman routing tables, and user IDs don't use IPs and one
device on the network could potentially be home to many user IDs.  The
decoupling of users and devices, making it impossible to track a user
back to a specific device, is by design.


## Usage

To use Ratman, you need to create a Router.  This type exposes an
async API to interact with various network abstractions.  A networking
endpoint and basic datagram types are defined in the `ratman-netmod`
crate, and the routing identities are defined in the `ratman-identity`
crate.


## Interface routing

The interface that binds the Ratman router to underlying drivers is
called `netmod`, which handles sending and receiving frames.  A frame
is a piece of data, with a checksum, which may be part of a larger
message.  In the qaul.net repository, you can find several driver
implementations for various platforms.  If you need to write your own,
don't hesitate to ask for help.

Routing is then done by mapping a user ID to an interface (plus some
target data that's left to the driver to interpret).  This way Ratman
is able to route across network boundaries, and on unpriviledged
hardware (such as phones).


## Clocking

Generally, Ratman handles scheduling and internal clocking for you.
There's no need to call update functions to make poll's work, or to
actually dispatch messages.  During initialisation the constructor
spawns several long running tasks, that deal with various tasks in the
router stack in a loop.  The downside to this is that the user of the
library (your app) has no control over how this code is called.

This is where the Router API adds clock points, and the `clock`
submodule, enabling you to reduce data rates in low power settings,
without having to teach Ratman about your platform specifics.


## License

Ratman is part of the qaul.net project, and licensed under the [GNU
Affero General Public License version 3 or
later](../licenses/agpl-3.0.md).

See the main qaul.net repository README for additional permissions
granted by the authors for this code.
