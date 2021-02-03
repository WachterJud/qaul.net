# qaul-linux - qaul.net linux client

This client is still in it's early stage. It provides the 


## Build & Run

The client is automatically built, when building qaul.net with cargo.

To run the client, you need to set the environment variables first

```
# set environment variables
export HUBD_PEERS=/PATH/TO/peers.txt
export HUBD_PORT=9001

# start 
qaul-linux -P peers.txt

```



## Network Modules

The following network modules are available:

* netmod-tcp: for Internet overlay networks.
* netmod-udp: to find clients in the local network.


## Configuration Options

Following is a list of qaul-hubd configuration values.  Those marked
with a `*` are mandatory.  Commandline arguments take precedence over
environment variables.

| ENV variable | Runtime argument | Description |
|----------------------|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `*` HUBD_PEERS=[PATH]    | -P / --peers [PATH] | Specify the path to a peer file, containing a newline-separated list of peers to connect to                                                                    |
| `*` HUBD_PORT=[PORT]     | -p / --port [PORT]  | Specify a tcp port to which qaul-hubd should bind itself to listen for incoming network traffic                                                                |
| HUBD_UDP_DISCOVERY=0 | --no-udp-discover   | Prevent qaul-hubd from registering a multicast address to find other clients on the same network.  Some networks may forbid this, or cause performance issues. |
| HUBD_SETUP_UPNP=0    | --no-upnp           | Disable automatic UPNP port forwarding.  Some networks may forbid this, or cause performance issues.                                                           |
| HUBD_RUN_MODE=[MODE] | -m / --mode [MODE]  | Specify the peering mode of this hub.  Possible values: "static", "dynamic"                                                                                    |
| HUBD_ADDR=[ADDR]     | -a / --addr [ADDR]  | A valid address to bind to.  Must be a valid ip address format. |


### Internet Overlay Network Peers

In order for the qaul.net instance to connect to an Internet overlay network. You can create a file with a new line separated list of IP addresses.

`peers.txt`

```
144.91.74.192:9001
95.216.98.55:11443
```
