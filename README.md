![http://qaul.net][img]

# qaul.net [![pipeline status]()][pipeline]

[img]: https://git.qaul.net/qaul/qaul.net/raw/release-1.0.0/doc/qaul-net.png
[pipeline]: https://gitlab.com/qaul/qaul-net/badges/master/pipeline.svg

**qaul.net** is an Internet independent ad-hoc wireless mesh-network 
suite that harnesses the power of everyday devices such as computers and 
smartphones to create a **Non-Centralized**, **Mesh Network** on which 
users can share files, have voice chats and send each other messages, 
however the power of qaul.net allows endless services over the service 
API. qaul.net removes the dependence on the centralized services such as 
the Internet and cellular networks and creates a network that anyone can 
be part of and share freely with no censorship what so ever.


## Development status

The project is currently being re-written for a more modular and portable 
approach. The new Release will be qaul.net 2.0. Please check 
our milestones & issues to get an idea of the development plan and 
status.
If you want to get involved, please [get in touch]()!

For the latest stable release, check the [`release-1.0.0`][release] branch.

[release]: https://github.com/qaul/qaul.net/tree/release-1.0.0


qaul.net 2.0 will have the following features:

* File sharing (both private and public)
* Voice calling
* Text messaging (both private and public)
* User discovery (who is in the network)
* Fully encrypted with a decentralized trust model
* Multi network mesh, using several mobile friendly connection modes
* Real time and decentralized mesh communication
* Extendable via service API


## Build Instructions

The project is being re-written in Rust, thus using [cargo][cargo] as a build system.
If you don't have Rust installed, you can get it [here](https://rustup.sh) or via your OS.


## License

qaul.net is free open source software licensed under the 
[GNU General Public License version 3](licenses/gpl-3.0.md).

To see all external code's licenses used in this project please 
visit the [`licenses` directory](licenses).
