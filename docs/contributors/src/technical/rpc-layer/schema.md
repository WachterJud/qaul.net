# Writing a wire schema

The qrpc bus is backed by the [capn proto] wire format.  It's similar
to Google's Protocol Buffers, since it was created by the primary
author of v2.  It improves in a few key ways, and while qrpc doesn't
use it's full feature set, it solves a lot of problems that would
otherwise be annoying and hard.

This does however mean that you will need to write a capn' proto
schema file for any time that is sent over the network, and you will
need to parse incoming types, check them for correctness (for example:
are all fields set that need to be set?), and then map their values to
functions and types in your API.

This also means that when removing parameters from functions, you need
to handle the case that older clients send them, and when adding
options, you need to be aware that older clients will need to fall
back to a default value.  Anything else would lead to a breaking
change in your RPC API.

