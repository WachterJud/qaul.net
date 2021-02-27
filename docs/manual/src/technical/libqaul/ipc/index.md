# libqaul IPC Interfaces

While libqaul and all of the qaul.net services are written in Rust,
that doens't mean that you need to write your application in Rust.
Especially considering that your application might want to connect to
an already running daemon on a system, your application needs to
connect to it in some way.

This is where the IPC interfaces come in (yes, ther's a few).

The [HTTP/json:api](http) interface was primarily written for the
webgui, but it also exposes the core libaul functions, so can be used
by your application.  Check out the external book for the API.

Additionally there are more interfaces that are documented in this
chapter.  Furthermore, if you think that there's a benefit in building
a new IPC interface for libqaul, get in touch! We're very curious.

[http]: docs.qaul.net/http-api/
