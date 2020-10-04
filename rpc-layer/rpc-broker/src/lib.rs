//! # qaul rpc-broker
//!
//! An extensible rpc message broker for the libqaul ecosystem.

mod socket;

/// Hold the main broker state
pub struct Broker {
    
}


/* 
Stuff I need

service -> service
service -> libqaul
libqaul -> service (reply, push/ subscription)


Each service has two parts: service core, and service client lib

service client lib:

- no logic
- defines the API and types with capn proto

service core:

- all the logic
- no types
- connects to the broker to advertise it's capabilities

Service advertisement

- name
- hash id
- capabilities

*/
