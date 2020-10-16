![](docs/banner.svg)

**qaul** is a decentralised networking project, aiming to create easy
to use solutions to ad-hoc wireless communication.  It supports many
common desktop operating systems (Linux, Windows, MacOS, BSD, ...),
and Android mobile phones.  iOS support is on the roadmap.

**qaul.net** is both a cross-platform application, implementing
**messaging**, **filesharing**, and **voice calls**, but also a
**development toolkit** to create fully decentralised third-party
applications.

In order to be able to run on unpriviledged mobile platforms qaul.net
implements **decentralised routing protocols** and utilities entirely
in userspace.  The codebase is largely written in
[Rust](https://rustlang.org), with only a few compatibility components
being written in more platform specific languages.

ALl parts of the project are contained in this repository.  Following
is an overview of the available components.  Each component has
additional information.

| Component   | Description      |
|-------------|------------------|
| [clients]   | qaul.net end-user applications for various platforms |
| [docs]      | Manuals (for both users and developers), and tools to build and deploy documentation |
| [emberweb]  | Cross-platform web interface bundled in with various user clients |
| [libqaul]   | Core library of the qaul.net ecosystem.  Provides networking abstractions, user management and discovery |
| [licenses]  | Set of license texts that are in use in this repository |
| [netmods]   | Platform-specific networking interface drivers |
| [nix]       | [nix](https://nixos.org) related build utilities |
| [ratman]    | A decentralised and modular userspace frame router |
| [rpc-layer] | qaul.net specific rpc system (qrpc) to support third-party components |
| [tests]     | Integrated test suite for various components.  Most of the code also has inline tests |
| [utils]     | Set of utilities that are used in various places and don't fit anywhere else |

[clients]: ./clients
[docs]: ./docs
[emberweb]: ./emberweb
[libqaul]: ./libqaul
[licenses]: ./licenses
[netmods]: ./netmods
[nix]: ./nix
[ratman]: ./ratman
[rpc-layer]: ./rpc-layer
[tests]: ./tests
[utils]: ./utils


## Overview

Most traditional networking infrastructure (both the transmission
layer, as well as applications) operate in a very centralised way.
Clients connect to servers, and devices to towers.  This makes it very
vulnerable to attacks.  Natural disasters or opressive governments can
easily shut down communication for millions of people, potentially
putting people at risk, and slowing down any activist movement.

qaul.net aims to solve this issue by creating decentralised circuits
between devices directly.  Additionally, it's routing approach takes
into account that connections are imperfect, and that a stable link
between two devices might not be impossible.  For this scenario the
network caches undelivered messages, carrying them towards their
destination until the recipient comes back online.

Routing in a qaul network is based on public keys, creating a large
enough address space for big communities.  Connecting devices together
happens via channel-specific drivers.  When creating a circuit,
roaming between various connection modes is common.

To learn more about the technical side of qaul.net, check out the
[contributors manual].

## How to use

There's no single way to use qaul.net.  Various platforms support
different clients, and a qaul network can consist of many different
components interacting with each other.  To get started, check out the
[user manual]!

[user manual]: https://docs.qaul.net/user


## Contributing

Social processes, code, and design guidelines are outlined in the
[contributors manual].  We have a developer chat hosted on [matrix];
we would be happy to answer any questions you have there.  For more
long-form posting we have a [mailing list].  We also accept patches
via e-mail!

If you want some inspiration for what you can do with qaul.net, check
out the [services] section.

[contributor manual]: https://docs.qaul.net/contributors
[matrix]: https://matrix.to/#/!ljaaylfsbkWFYNoNPT:fairydust.space?via=fairydust.space&via=matrix.org&via=public.cat
[mailing list]: https://lists.sr.ht/~qaul/community
[services]: ./services

## License

qaul.net is free and open source software licensed under the [GNU
Affero General Public License version 3 or
later](licenses/agpl-3.0.md).

**Additional Permissions:** For Submission to the Apple App Store:
Provided that you are otherwise in compliance with the AGPLv3 for each
covered work you convey (including without limitation making the
Corresponding Source available in compliance with Section 6 of the
AGPLv3), the qaul.net developers also grant you the additional
permission to convey through the Apple App Store non-source executable
versions of the Program as incorporated into each applicable covered
work as Executable Versions only under the Mozilla Public License
version 2.0.

A copy of both the AGPL-3.0 and MPL-2.0 license texts are included in
this repository, along other external licenses for third-party code,
and can be found in the [licenses](licenses) directory.
