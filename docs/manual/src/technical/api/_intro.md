# Development Interfaces

This document will change quite a lot during the development,
especially because there are several layers of interfaces that need to
be documented.  In the future we might want to move some of them into
their dedicated books.

For now, this document is an outline for the json RPC interface of
`libqaul`.  It's being made available via the `libqaul-http` crate.


## Envelope

Every message is packed into an envelope.  It contains metadata that
might be present in some transports, but not others.  The envelope was
generally built to be streamed via sockets, where message association
isn't handled via the transport protocol (like http).

```json
{
  "id": "1",
  "auth": {
    "id": "7F9B BAA9 DF90 0F48 CD41 7FD4 2C9C E432 1309 FB79 8CF9 C56F FB9A 8F2C 29C1 12D9",
    "token": "56C0 F4C0 BD0D E822 B21D 4120 CC43 5C73 1F92 B246 C988 3335 72F5 8F1A 2701 3FD7"
  },
  "page": "1",
  "method": "query",
  "kind": "messages",
  "data": {
    "query": {
      "tag": {
        "key": "room-id",
        "val": "A88A 7CC6 DF68 CE60 2D63 64CB 7393 8924 653D 52FB DBC6 2A39 D892 A9BD 6FA4 FD5D"
      }
    },
    "service": "net.qaul.chat"
  }
}
```

This type can be found in `libqaul/rpc/src/json/mod.rs`.  The general
structure of requests stays the same: there's an ID, some auth data,
the page (which isn't implemented yet), `method` and `kind` the
request operates on.


There are several types available (`kind`):

- **users**, either a local or remote user
- **contacts**, some user-specific contact data for another user
- **chat-rooms**, a chat room via service (requires `qaul` feature)
- **chat-messages**, a chat room message (requires `qaul` feature)


Possible `methods` are (not all combinations exist though!):

- **list**, get a list of all, if available
- **create**, create a new (with side-effects, such as sending)
- **delete**, remove from local stores
- **get**, return single item by ID
- **query**, return a set of items, according to some query
- **modify**, change an existing item in some way
- **subscribe**, (not implemented yet)
- **unsubscribe**, (not implemented yet)


Three special `methods` only exist for users: 

- **login**, validate password and receive auth tokens
- **logout**, end session token
- **repass**, change user passphrase


Following is a list of examples of how to construct valid requests.
If in doubt, the code that parses these is in
`libqaul/rpc/src/json/parser.rs`.  You can always look it up there.
