<h1 align="center">Tide</h1>
<div align="center">
 <h1> قول </h1>
</di>

<br />



![http://qaul.net/](https://git.open-communication.net/qaul/qaul.net/raw/release-1.0.0/doc/qaul-net.png)

# qaul.net [![pipeline status](https://git.open-communication.net/qaul/qaul.net/badges/master/pipeline.svg)](https://git.open-communication.net/qaul/qaul.net/commits/master)

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
