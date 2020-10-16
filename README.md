<style>
h1 { font-size: 5rem; margin: 1em 0 0;}
.subtitle { padding: 0; margin: 0 6rem; }
</style>

<h1 align="center">قول</h1>
<div class="subtitle"><strong>qaul.net</strong> — tools for the next revolation</div>
<h1 align="center"> قول - qaul.net</h1>
<div align="center">
	<strong>decentralised ad-hoc wireless communication</strong>
</div>

<br />

qaul.net is a decentralised networking project, aiming to create
easier to use solutions to ad-hoc wireless communication.  It supports
many common desktop operating systems (Linux, Windows, MacOS, BSD,
...), and Android mobile phones.  iOS is a desired target, but isn't
yet implemented.

qaul.net is both a cross-platform end-user application, providing
messaging, filesharing, and voice calls, but also a toolkit to create
fully decentralised third-party applications.

In order to be able to run on unpriviledged mobile platforms qaul.net
implements decentralised routing protocols and utilities entirely in
userspace.  The codebase is largely written in Rust, with only a few
compatibility components being written in more platform specific
languages.

The project is entirely contained in this repository.  Following is an
overview of the available components.

| Component | Description      |
|-----------|------------------|
| [clients]    | qaul.net end-user applications for various platforms |
| [docs]    | Contains manuals (for users and developers), and tools to build various pieces of documentation |
| [emberweb] | Cross-platform web interface bundled into  various user clients |
| [libqaul] | Primary library of the qaul.net ecosystem.  Provides network messaging abstractions, user session management and discovery |
| [licenses] | Set of license texts that are in use in this repository |
| [netmods] | Platform-specific networking interface drivers |
| [nix] | [nix](https://nixos.org) related build utilities |
| [ratman] | A decentralised userspace packet router |
| [rpc-layer] | qaul.net specific rpc system (qrpc) to support third-party components |
| [tests] | Integrated test suite for various components.  Most of the code also has inline tests |
| [utils] | Set of utilities that are used in various places and don't fit anywhere else |

[clients]: ./clients
[docs]: ./docs
[emberweb]: ./emberweb
[libqaul]: ./libqaul
[licenses]: ./licenses
[netmods]: ./netmods
[nix]: ./nix
[ratman]: ./ratman
[rpc]: ./rpc
[tests]: ./tests
[utils]: ./utils

It supports instant messaging, voice calls, social media, radio
broadcast, and file sharing — all in a single app. Additionally it is
a suite of free software libraries, meaning that anyone can study and
modify it, and use it to create their own apps that can operate on a
qaul network, independent of the internet.

**qaul.net** is an Internet independent ad-hoc wireless mesh-network
suite that harnesses the power of everyday devices such as computers
and smartphones to create a **Non-Centralized**, **Mesh Network** on
which users can share files, have voice chats and send each other
messages, however the power of qaul.net allows endless services over
the service API. qaul.net removes the dependence on the centralized
services such as the Internet and cellular networks and creates a
network that anyone can be part of and share freely with no censorship
what so ever.


## Development status

The project is currently being re-written for a more modular and
portable approach. The new Release will be qaul.net 2.0. Please check
our [milestones] & [issues] to get an idea of the development plan and
status. If you want to get involved, see how to [participate] and read 
the [contributors-guide].

For the latest stable release, check the [`release-1.0.0`][release]
branch.

[milestones]: https://git.open-communication.net/groups/qaul/-/milestones
[issues]: https://git.open-communication.net/qaul/qaul.net/issues
[participate]: https://qaul.net/#participation
[contributors-guide]: https://docs.qaul.net/contributors/
[release]: https://git.open-communication.net/qaul/qaul.net/tree/release-1.0.0


## Build Instructions

The qaul.net project has many libraries and clients, for different
platforms.  Check the "clients" directory for instructions on how to
build them.  Because some platforms require some bootstrapping you may
have to build different parts in sequence: we don't currently have an
overarching build system for this.

To build the rust libraries for most platforms, simply run `cargo
build --release` (for release mode).  To build android, check the
[`build.sh`](./clients/android/build.sh) in that client.  The web UI
is built with emberJS and con be found [here](webgui).

To build the web stack on Linux, you can build the ember UI with
`ember dist`, then move the output to `libqaul/http/ui`, so that they
can be included in the web server, which will then serve them via
`clients/linux-http-test` or `clients/android`.

The repo has a `shell.nix` if you want to use nix to get dependencies,
however this doesn't actually build the project.


## Documentation

Documentation is available on [docs.qaul.net](https://docs.qaul.net).


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
