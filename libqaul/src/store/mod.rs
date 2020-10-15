//! libqaul specific storage wrappers
//!
//! The database and storage library in use is called alexandria,
//! which encodes data as diffs.  This module is repsonsible for
//! making Diffs and records map onto each other.  Each operation on a
//! type yields in a diff that the storage system can then apply, and
//! reading a data type from a record.

mod messages;
pub(crate) use messages::MsgUtils;

mod services;

mod users;
pub(crate) use users::KeyWrap;

use crate::{messages::SigTrust, Identity};
use alexandria::record::{kv::Value, RecordRef};
use std::collections::{BTreeMap, BTreeSet};

/// A utility trait to get around the orphan rules
pub trait FromRecord<T> {
    fn from_rec(rec: RecordRef) -> T;
}

struct Conv;

impl Conv {
    pub(self) fn id(v: &Value) -> Identity {
        match v {
            Value::Vec(vec) => Identity::from_bytes(vec),
            v => panic!("Invalid conversion: {:?} -> Id", v),
        }
    }

    pub(self) fn string(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            v => panic!("Invalid conversion: {:?} -> String", v),
        }
    }

    pub(self) fn map(v: &Value) -> BTreeMap<String, String> {
        match v {
            Value::Map(map) => map
                .iter()
                .map(|(k, v)| match v {
                    Value::String(s) => (k.clone(), s.clone()),
                    v => panic!("Invalid map-inner conversion: {:?} -> String", v),
                })
                .collect(),
            v => panic!("Invalid conversion: {:?} -> BTreeMap<String, String>", v),
        }
    }

    pub(self) fn bin_map(v: &Value) -> BTreeMap<String, Vec<u8>> {
        match v {
            Value::Map(map) => map
                .iter()
                .map(|(k, v)| match v {
                    ref v @ Value::Vec(_) => (k.clone(), Self::binvec(v)),
                    v => panic!("Invalid map-inner conversion: {:?} -> Vec<u8>", v),
                })
                .collect(),
            v => panic!("Invalid conversion: {:?} -> BTreeMap<String, String>", v),
        }
    }

    pub(self) fn str_set(v: &Value) -> BTreeSet<String> {
        match v {
            Value::List(list) => list
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    v => panic!("Invalid set-inner conversion: {:?} -> String", v),
                })
                .collect(),
            v => panic!("Invalid conversion: {:?} -> BTreeSet<String>", v),
        }
    }

    pub(self) fn binvec(v: &Value) -> Vec<u8> {
        match v {
            Value::Vec(v) => v.clone(),
            v => panic!("Invalid conversion: {:?} -> Vec<u8>", v),
        }
    }

    pub(self) fn sig_trust(v: &Value) -> SigTrust {
        match v {
            Value::String(s) => match s.as_str() {
                "trusted" => SigTrust::Trusted,
                "unverified" => SigTrust::Unverified,
                "invalid" => SigTrust::Invalid,
                v => panic!("Invalid string payload: '{}'", v),
            },
            v => panic!("Invalid conversion: {:?} -> SigTrust", v),
        }
    }
}
