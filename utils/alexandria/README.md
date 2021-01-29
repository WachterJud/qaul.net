# alexandria 📚 [![][irc-badge]][irc-url]

[irc-badge]: https://img.shields.io/badge/IRC-%23qaul.net-1e72ff.svg
[irc-url]: https://www.irccloud.com/invite?channel=%23qaul.net&hostname=irc.freenode.org&port=6697&ssl=1


Strongly typed, embedded record database with seamless encryption at
rest storage.  Supports key-value Diff transactions, as well as
externally loaded binary payloads.  Supports encrypted metadata
without extra configuration.

Alexandria has the following features:

- Store data on internal db path
- Query the database by path or dynamic search tags
- Subscribe to events based on query
- Iterate over query dynamically
- Store data in session or global namespaces

**Notice:** alexandria should be considered experimental and not used
in production systems where data loss is unacceptable.


## How to use

Alexandria requires `rustc` 1.42 to compile.

```rust
use alexandria::{Library, Builder};
use tempfile::tempdir();

let dir = tempdir().unwrap();
let lib = Builder::new()
              .offset(dir.path())
              .root_sec("car horse battery staple")
              .build()?


```


Alexandria is developed as part of [qaul.net][website]. We have a
[mailing list][list] and an [IRC channel][irc]! Please come by and ask
us questions!  (the issue tracker is a bad place to ask questions)

[website]: https://qaul.net
[list]: https://lists.sr.ht/~qaul/community/
[irc]: https://irccloud.com/freenode/#qaul.net


## License

Alexandria is free software and part of [qaul.net][qaul.net]. You
are free to use, modify and redistribute the source code under the
terms of the GNU General Public License 3.0 or (at your choice) any
later version. For a full copy of the license, see `LICENSE` in the
source directory attached.

**Additional Permissions:** For Submission to the Apple App Store:
Provided that you are otherwise in compliance with the GPLv3 for each
covered work you convey (including without limitation making the
Corresponding Source available in compliance with Section 6 of the
GPLv3), the qaul.net developers also grant you the additional
permission to convey through the Apple App Store non-source executable
versions of the Program as incorporated into each applicable covered
work as Executable Versions only under the Mozilla Public License
version 2.0.

A copy of both the GPL-3.0 and MPL-2.0 license texts are included in
this repository.
