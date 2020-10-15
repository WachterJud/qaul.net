//! Alexandria tagging system
//!
//! The internals of alexandria work with records and search tags.
//! Records are fully encrypted, meaning that searching them in a
//! reasonable amount of work is impossible.  To solve this problem
//! Alexandria uses search tags, that are searched in a separately
//! encrypted tag tables.  This way records can be searched via tags,
//! without exposing their data bodies.

use serde::{
    de::{Error, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::BTreeSet, fmt};

/// A set of tags where every tag is unique
///
/// Simply construct a set via one of the `From` implementations of a
/// containing type.
///
/// ```norun
/// # use alexandria::data::TagSet;
/// # use std::collections::BTreeSet;
/// let _: TagSet = vec![].into();
/// let _: TagSet = BTreeSet::default().into();
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TagSet(BTreeSet<Tag>);

impl TagSet {
    pub fn empty() -> Self {
        Self(Default::default())
    }

    pub fn insert(&mut self, t: Tag) {
        self.0.insert(t);
    }

    /// Merge this and another tagset together into one
    pub fn merge<T>(self, tags: T) -> Self
    where
        T: Into<Self>,
    {
        let ts = tags.into();
        Self(ts.0.into_iter().fold(self.0, |mut set, t| {
            set.insert(t);
            set
        }))
    }

    pub fn remove(&mut self, t: &Tag) {
        self.0.remove(t);
    }

    pub fn contains(&self, t: &Tag) -> bool {
        self.0.contains(t)
    }

    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Any overlap between `self` and `o`
    pub fn intersect(&self, o: &TagSet) -> bool {
        o.iter().fold(false, |acc, t| acc || self.0.contains(t))
    }

    /// A subset where `o` needs to be contained entirely in `self`
    pub fn subset(&self, o: &TagSet) -> bool {
        o.iter().fold(true, |acc, t| acc && self.0.contains(t))
    }

    /// An equality set where `o` and `self` are the same
    pub fn equality(&self, o: &TagSet) -> bool {
        self.0 == o.0
    }

    /// No overlay between `self` and `o`
    pub fn not(&self, o: &TagSet) -> bool {
        o.iter().fold(true, |acc, tag| acc && !self.0.contains(tag))
    }

    /// Return an iterator over the inner collection
    pub fn iter(&self) -> impl Iterator<Item = &Tag> {
        self.0.iter()
    }
}

impl From<Tag> for TagSet {
    fn from(t: Tag) -> Self {
        Self::from(vec![t])
    }
}

impl<'tag> From<&'tag Tag> for TagSet {
    fn from(t: &'tag Tag) -> Self {
        Self::from(vec![t.clone()])
    }
}

impl From<Vec<Tag>> for TagSet {
    fn from(vec: Vec<Tag>) -> Self {
        Self(vec.into_iter().fold(BTreeSet::new(), |mut set, tag| {
            set.insert(tag);
            set
        }))
    }
}

impl From<&Vec<Tag>> for TagSet {
    fn from(vec: &Vec<Tag>) -> Self {
        Self(vec.iter().fold(BTreeSet::new(), |mut set, tag| {
            set.insert(tag.clone());
            set
        }))
    }
}

impl From<Vec<&Tag>> for TagSet {
    fn from(vec: Vec<&Tag>) -> Self {
        Self(vec.into_iter().fold(BTreeSet::new(), |mut set, tag| {
            set.insert(tag.clone());
            set
        }))
    }
}

impl From<BTreeSet<Tag>> for TagSet {
    fn from(set: BTreeSet<Tag>) -> Self {
        Self(set)
    }
}

impl From<TagSet> for BTreeSet<Tag> {
    fn from(ts: TagSet) -> Self {
        ts.0
    }
}

/// A generic metadata tag
///
/// Because searching through message or file payloads might be slow,
/// and I/O intensive (especially within the secret store), all
/// records types have a tag metadata interface.  It's up to the
/// implementor of an application to use these to search records,
/// create relationships, or subsets of data.
///
/// Tags can also be serialised to either a human readable or binary
/// format.  To disable this, re-compile alexandria without the
/// `human-tags` or `tag-serialize` flags
#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Tag {
    /// A string key for a tag
    pub key: String,
    /// Some binary data that is up to a service to interpret
    pub val: Vec<u8>,
}

impl Tag {
    /// Create a new MsgTag with key and value
    pub fn new<K, I>(key: K, val: I) -> Self
    where
        K: Into<String>,
        I: IntoIterator<Item = u8>,
    {
        Self {
            key: key.into(),
            val: val.into_iter().collect(),
        }
    }

    /// Create a tag that consists of only a key, with no value
    pub fn empty<K>(key: K) -> Self
    where
        K: Into<String>,
    {
        Self::new(key, vec![])
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
struct HumanVec(Vec<u8>);

impl Serialize for HumanVec {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if ser.is_human_readable() {
            ser.serialize_str(
                &hex::encode_upper(&self.0)
                    .as_bytes()
                    .chunks(4)
                    .map(std::str::from_utf8)
                    .collect::<Result<String, _>>()
                    .unwrap(),
            )
        } else {
            ser.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for HumanVec {
    fn deserialize<D>(der: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HumanVecVis;

        impl HumanVecVis {
            fn from_str<E: Error>(s: &str) -> Result<HumanVec, E> {
                Self::from_bytes(&hex::decode(s).map_err(|e| E::custom(e))?)
            }

            fn from_bytes<E: Error, V: AsRef<[u8]>>(v: V) -> Result<HumanVec, E> {
                let v = v.as_ref();
                Ok(HumanVec(v.iter().cloned().collect()))
            }
        }

        impl<'de> Visitor<'de> for HumanVecVis {
            type Value = HumanVec;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "A byte array or a hex string encoded byte array",)
            }

            fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Self::from_str(v)
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Self::from_str(&v)
            }

            fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
                Self::from_bytes(v)
            }

            fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                Self::from_bytes(v)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element::<u8>()? {
                    v.push(b);
                }

                Self::from_bytes(v)
            }
        }

        if der.is_human_readable() {
            der.deserialize_str(HumanVecVis)
        } else {
            der.deserialize_bytes(HumanVecVis)
        }
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        dbg!();
        let mut state = ser.serialize_struct("Tag", 2)?;
        state.serialize_field("key", &self.key)?;
        state.serialize_field("val", &HumanVec(self.val.clone()))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(der: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Responsible for deserialising hex-encoded payloads
        ///
        /// This visitor is called when the deserialiser is working
        /// for a human readable format, such as json.
        struct TagVisitor;

        impl<'de> Visitor<'de> for TagVisitor {
            type Value = Tag;

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let key: String = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;

                let hvec: HumanVec = seq
                    .next_element()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?;
                let val: Vec<u8> = hvec.0;

                Ok(Tag { key, val })
            }

            // json will try to deserialize structs as maps
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut key: Option<String> = None;
                let mut value: Option<HumanVec> = None;

                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() {
                        "key" => {
                            if key.is_some() {
                                return Err(Error::duplicate_field("key"));
                            }
                            key = Some(map.next_value()?);
                        }
                        "val" => {
                            if value.is_some() {
                                return Err(Error::duplicate_field("val"));
                            }
                            value = Some(map.next_value()?);
                        }
                        f => {
                            static FIELDS: &'static [&'static str] = &["key", "val"];
                            return Err(Error::unknown_field(f, FIELDS));
                        }
                    }
                }

                let key = key.ok_or_else(|| Error::missing_field("key"))?;
                let value = value.ok_or_else(|| Error::missing_field("val"))?;

                Ok(Tag { key, val: value.0 })
            }

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                fmt.write_str("struct Tag { key, val }")
            }
        }

        der.deserialize_struct("Tag", &["key", "val"], TagVisitor)
    }
}

#[test]
fn serialize_tag_json() {
    let t = Tag {
        key: "blorp".into(),
        val: vec![172, 171],
    };

    use serde_json;
    let json = serde_json::to_string(&t).unwrap();
    assert_eq!(json.as_str(), r#"{"key":"blorp","val":"ACAB"}"#);
}

#[test]
fn serialize_tag_bincode() {
    let t = Tag {
        key: "blorp".into(),
        val: vec![172, 171],
    };

    use bincode;
    let bc = bincode::serialize(&t).unwrap();
    assert_eq!(
        bc.as_slice(),
        &[5, 0, 0, 0, 0, 0, 0, 0, 98, 108, 111, 114, 112, 2, 0, 0, 0, 0, 0, 0, 0, 172, 171]
    );
}

#[test]
fn deserialize_tag_json() {
    use serde_json;
    let json = serde_json::json!( {
        "key": "blorp",
        "val": "ACAB",
    });
    let t: Tag = serde_json::from_value(json).unwrap();

    assert_eq!(
        t,
        Tag {
            key: "blorp".into(),
            val: vec![172, 171],
        }
    );
}

#[test]
fn deserialize_tag_bincode() {
    let bin = [
        5, 0, 0, 0, 0, 0, 0, 0, 98, 108, 111, 114, 112, 2, 0, 0, 0, 0, 0, 0, 0, 172, 171,
    ];

    use bincode;
    let t: Tag = bincode::deserialize(&bin).unwrap();

    assert_eq!(
        t,
        Tag {
            key: "blorp".into(),
            val: vec![172, 171],
        }
    );
}

#[test]
fn subset_1() {
    let whole = TagSet::from(vec![Tag::empty("a"), Tag::empty("b")]);
    let sub = TagSet::from(vec![Tag::empty("a")]);
    assert!(whole.subset(&sub));
}

#[test]
fn subset_2() {
    let a = TagSet::from(vec![Tag::empty("a"), Tag::empty("b")]);
    let b = TagSet::from(vec![Tag::empty("b"), Tag::empty("c")]);
    assert!(!a.subset(&b));
}

#[test]
fn intersect_1() {
    let a = TagSet::from(vec![Tag::empty("a"), Tag::empty("b")]);
    let b = TagSet::from(vec![Tag::empty("b"), Tag::empty("c")]);
    assert!(a.intersect(&b));
}

#[test]
fn intersect_4() {
    let a = TagSet::from(vec![Tag::empty("a"), Tag::empty("b"), Tag::empty("c")]);
    let b = TagSet::from(vec![Tag::empty("d"), Tag::empty("e")]);
    assert!(!a.intersect(&b));
}

#[test]
fn not_1() {
    let a = TagSet::from(vec![Tag::empty("a"), Tag::empty("b"), Tag::empty("c")]);
    let b = TagSet::from(vec![Tag::empty("d"), Tag::empty("e")]);
    assert!(a.not(&b));
}
