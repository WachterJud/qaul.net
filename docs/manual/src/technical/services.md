# qaul.net services

Following is a list of services that are bundled with qaul.net, and
what they can do for you.

Services have an internal ID that is used to identify them in
`libqaul`, using reverse fully qualified domain specifiers (for
example `net.qaul.my-app`).


## Fundamental services

As outlined in the technical introduction, some services are
fundamental, meaning they provide functionality to other services,
instead of being user-facing.


### Type Messages

This service provides an interface to send strongly typed messages,
with extension hooks for external services to provide their own type
data.  In `libqaul` all payloads are raw binary which is done to save
space, and not complicate the API unecessarily.  However sometimes
type information is useful, and an app service might want to rely on
it.  That's what this fundamental service is for!

* Service ID: `net.qaul.fundamental.messages`
* Crate name: `qaul-type-messages`


### Group abstractions

This service provides a group management abstraction.  While many app
services might want to manage groups, doing so in an encrypted system,
with no direct group leader can be tricky.  To avoid having to
re-implement this more than once, all qaul.net bundled app services
use this fundamental service, and it is recommended that your app
services do too!

* Service ID: `net.qaul.fundamental.groups`
* Crate name: `qaul-groups`


## App services

These are services that provide some user-facing functionality.  Some
will ship with their own UI, while others will use a UI bundle (such
as the main qaul.net services).


### Feed

* Service ID: `net.qaul.feed`
* Crate name: `qaul-feed`

Public message feed, similar to micro-blogging applications such as
mastodon or twitter.  Users can compose short-medium length messages
that get shared with the whole network.  You have the choice of
filtering by "all messages" and "only following", where you can only
display messages sent by friends or people you trust.


### Messaging

* Service ID: `net.qaul.messaging`
* Crate name: `qaul-messaging`

Private (1-to-1 or groups) text messaging, that can optionally (by
default) be encrypted. Conversations can either be displayed as a feed
(like chat), or threaded (like e-mail).  Files can be sent either
in-line (if the payload is small enough), or via file-announce
messages that use the "files" service.


### Filesharing

* Service ID: `net.qaul.files`
* Crate name: `qaul-files`

Filesharing via announce-links similar to how torrents get announced
on trackers.  You can announce a file to either a group of people, a
singe user, or the whole network.  Optionally this service can be
configured to download all files that were publicly announced to allow
servers to replicate a "public archive" that users can have access to,
if the original source of a file disappears.


### Voices

* Service ID: `net.qaul.voices`
* Crate name: `qaul-voices`

Integrates with various platform features to allow voice call
streaming over the qaul.net, as well as preparing audio messages that
can be inlined into messages, or sent to many people via announce
links.


### Radio

* Service ID: `net.qaul.radio`
* Crate name: `qaul-radio`

Similarly to voices, it integrates into platform features to provide
audio capture and playback, but for one-to-many streams.  This way
people can broadcast themselves into the network, while others can
tune into a program, without being able to respond.  Similar to files,
this service can be configured to automatically archive radio
broadcasts for community servers.
