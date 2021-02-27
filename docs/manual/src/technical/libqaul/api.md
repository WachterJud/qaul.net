# Service API

This section of the manual covers the service API.  Please check the
`libqaul` Rust API docs for actual function docs.  This document will
go into the concepts more than actual code.

The API is written in Rust and uses `async/await` futures, built with
the `async-std` runtime.  If you build a service in Rust, we recommend
you use the same runtime to keep the application binary as small as
possible.

Code and function docs for libqaul can be found [here][libqaul]!

[libqaul]: https://docs.qaul.net/api/libqaul/index.html

## Users

At the heart of almost every call to the service API is a user
session.  You can use the `login` and `logout` functions to
manipulatio a session.  The user authentication object, containing the
user's ID, and a token assigned with your session, is required for
every subsequent call into the API.  Tokens will also expire with
inactivity, so building your code to be resistent to `Error::NoAuth`
errors is always a good idea.

Attached to a user comes a message, contacts and file store.  A global
user store also exists.  These are provided by function scope
endpoints, that encapsulate various functions in a type/namespace to
make interacting with them easier.


## User store

An instance of libqaul keeps track of users on the network, and
general user announcements, which are saved in the user store.  These
user profiles are slowly filled with metadata, as more information
becomes available about a user: did they set an avatar recently, do
they have a preferred nickname or pronouns, etc.

It's also possible to search by any attribute in the store, even
optional user generated fields like location.


## Mesage store

For each user, a message store overlay is kept, meaning that an
instance only keeps one copy of the actual messages, but what user has
access to them changes what part of the store they can see.  Similar
to the user store, it's possible to send queries to the store to get
messages sent to the current user and service, according to a
parametric search.


## Contact book

Each user also keeps a private contact overlay, in which they can
annotate users they have interacted with.  Available fields are trust,
if two users have met, and free-form additional metadata.


## Storage

Because libqaul already has a mechanism in place to encrypt data for a
user at rest, this mechanism is exposed in the API to external
services.  On one hand this API can be used to store additional
metadata for a service, that is required to make it all work, or to
get files that the user has discovered on the network and that were
downloaded previously.

This API scope is complemented by the "files" endpoint, that provides
a way to announce, pack, update and send files into a network.
**Note** maybe it would make more sense to join "files" and "storage",
and move the "files" functionality entirely into the files service.
