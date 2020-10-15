//! Handling service store (metadata) interaction with Alexandria

use super::{Conv, FromRecord};
use crate::services::MetadataMap;
use alexandria::{
    record::{kv::Value, RecordRef},
    utils::Diff,
};
use std::collections::BTreeMap;

const NAME: &'static str = "name";
const MAP: &'static str = "map";

impl FromRecord<MetadataMap> for MetadataMap {
    fn from_rec(rec: RecordRef) -> Self {
        let kv = rec.kv();

        Self::from(
            Conv::string(kv.get(NAME).unwrap()),
            Conv::bin_map(kv.get(MAP).unwrap()),
        )
    }
}

pub(crate) trait MetadataExt {
    fn init_diff(&self) -> Vec<Diff>;
    fn gen_diffset(&self, prev: &Self) -> Vec<Diff>;
}

impl MetadataExt for MetadataMap {
    fn init_diff(&self) -> Vec<Diff> {
        vec![
            Diff::map().insert(NAME, self.name().as_str()),
            Diff::map().insert(
                MAP,
                self.iter()
                    .map(|(k, v)| (k.clone(), Value::Vec(v.clone())))
                    .collect::<BTreeMap<String, Value>>(),
            ),
        ]
    }

    /// Generate a diffset based on the previous version of the map
    fn gen_diffset(&self, prev: &Self) -> Vec<Diff> {
        let mut vec = vec![];

        self.iter().for_each(|(key, val)| {
            match prev.get(key) {
                // If the key was present in the previous map, generate an update if the value has changed
                Some(prev) if prev != val => {
                    vec.push(Diff::map().nested(MAP, Diff::map().update(key, val.clone())));
                }
                // And if it wasn't we insert it normally
                None => {
                    vec.push(Diff::map().nested(MAP, Diff::map().insert(key, val.clone())));
                }
                _ => {}
            }
        });

        // Do another run in reverse and delete all keys that are now missing
        prev.iter().for_each(|(key, _)| {
            if self.get(key).is_none() {
                vec.push(Diff::map().nested(MAP, Diff::map().delete(key)));
            }
        });

        vec
    }
}

#[test]
fn metadata_diff_empty() {
    let m = MetadataMap::new("test");
    let diffs = m.init_diff();
    assert_eq!(diffs.len(), 2);
}

#[test]
fn metadata_diff_simple() {
    let m = MetadataMap::from(
        "test",
        vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])],
    );
    let diffs = m.init_diff();
    assert_eq!(diffs.len(), 2);
}

#[test]
fn metadata_diff_delete() {
    let old = MetadataMap::from(
        "test",
        vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])],
    );

    let new = MetadataMap::from("test", vec![("acab", vec![1, 3, 1, 2])]);

    let diffs = new.gen_diffset(&old);
    assert_eq!(diffs.len(), 1);
}

#[test]
fn metadata_diff_delete_update() {
    let old = MetadataMap::from(
        "test",
        vec![("key", vec![1, 2, 3, 4]), ("acab", vec![1, 3, 1, 2])],
    );

    let new = MetadataMap::from("test", vec![("acab", vec![13, 12])]);

    let diffs = new.gen_diffset(&old);
    assert_eq!(diffs.len(), 2);
}
