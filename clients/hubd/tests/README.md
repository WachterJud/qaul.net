# Test qaul-hubd

## Test qaul-hubd Locally

Run two instances on one machine.
Execute the scripts `local1.sh` and `local2.sh` in two terminals on the same computer.

`local1.sh` will start a qaul-hubd instance listening on port 9000.
This instance is configured dynamically, so it will accept connections from every machine

`local2.sh` will start another qaul-hubd instance listening on port 9001.
It connects to instance 1, as 127.0.0.1:9000 is referenced in it's peers list.
This instance is configured statically, this instance will not accept from IP's that are not in it's peers list.

```sh
## first start local1.sh in Terminal 1
./local1.sh

## then start local2.sh in Terminal 2
./local2.sh
```
