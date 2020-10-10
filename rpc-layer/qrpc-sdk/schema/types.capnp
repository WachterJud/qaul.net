@0xd20f567e3e118ea6;

# Type definitions for the qrpc-sdk

# The SDK handles the creation and management of services, and as such the
# Service type is the only real data type provided by this layer.  It's used by
# all components to register themselves and abstract away other components on
# the bus.
struct Service {
     # Names follow reverse FQD specification: net.qaul.my_app
     name @0 :Text;
     # Versions are up to services to interpret.  A simple
     # incrementing number is fine, but feel free to encode semver
     # into this field.
     version @1 :Int16;
     # A description of the service shown to end-users.  Make it something concise
     description @2 :Text;
}

