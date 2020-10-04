# qrpc-sdk

The qrpc protocol connections an rpc-broker with different service
endpoints.  Each service endpoint provides a set of callable
functions, and type serialisation data.


```
 Your app logic    Serialise types    Pass data along
+--------------+   +--------------+   +--------------+
| Your service | - |   qrpc-sdk   | - |  qrpc-broker |
+--------------+   +--------------+   +--------------+
                                              |
                   +--------------+   +--------------+   +--------------+
                   | Your UI app  | - |   qrpc-sdk   | - |    libqaul   | 
                   +--------------+   +--------------+   +--------------+
                     Your app UI      Deserialise types    Main db/ router
```

You can find more information in the [contributors manual][m].


[m]: https://docs.qaul.net/contributors/technical
