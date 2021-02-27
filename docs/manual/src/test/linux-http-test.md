# Test qaul.net on Linux with *linux-http-test* Executable

**Executable to test the qaul.net application with it's WebGui on linux.**

## Prerequisites

To be able to run the test you need to build the following:

* [Build the `linux-http-test` executable](../install/linux.md)
* [Build the ember WebGUI](../technical/webgui/install.md)


## Test

From the qaul.net project folder you can invoke the following command to start the test

```bash
# start linux-http-test with the path to the built webui content as parameter.
target/debug/linux-http-test webgui/dist
# or alternatively
cargo run --bin linux-http-test webgui/dist
```


Now you can now open the WebGUI in your web browser via [http://127.0.0.1:9900](http://127.0.0.1:9900)
