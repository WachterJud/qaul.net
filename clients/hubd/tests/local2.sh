#!/usr/bin/env bash

## start the `qaul-hubd` daemon
## execute this start script from within this folder after `cargo build`

../../../target/debug/qaul-hubd --no-upnp --mode dynamic --peers local2-peers.txt  --port 9001
