//! # A Ratman network identity abstraction
//!
//! Because Ratman is a userspace router with no concept of link layer
//! identities, network IDs are chosen to be fixed size byte arrays.
//! It's left to the implementing application to map these to some
//! useful source of identity.  This crate also provides a hashing
//! constructor behind the `digest` feature flag which can be used to
//! hash a secret to derive the identity value.
//!
//! Whatever scheme is chosen, two principles about identity must not
//! be violated:
//!
//! 1. There are no identity collisions
//! 2. Identities don't change mid-route
//!
//! This crate is part of the qaul.net project.  The docs for this
//! crate are probably lacking because currently Ratman/ libqaul are
//! the only users of it.  If you have questions, don't hesitate to
//! [contact us]!
//!
//! [contact us]: https://docs.qaul.net/manual/social/_intro.html

use cfg_if;
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    Deserialize, Serialize, Serializer,
};
use std::{
    fmt::{self, Debug, Display, Formatter},
    string::ToString,
};

cfg_if::cfg_if! {
    if #[cfg(features = "aligned")] {
        use std::mem::size_of,
        /// Length of the identity buffer to align with platform words
        pub const ID_LEN: usize = size_of::<usize>();
    } else {
        /// Length of the identity buffer to align with an ed25519 pubkey
        pub const ID_LEN: usize = 32;
    }
}

/// A generic object identifier
#[derive(Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct Identity([u8; ID_LEN]);

impl Debug for Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<ID: {}>", hex::encode_upper(self))
    }
}

impl Display for Identity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            hex::encode_upper(self)
                .as_bytes()
                .chunks(4)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
                .join("-")
        )
    }
}

impl Identity {
    /// Create an identity from the first 16 bytes of a vector
    ///
    /// This function will panic, if the provided vector isn't long
    /// enough, but extra data will simply be discarded.
    pub fn truncate<'vec, V: Into<&'vec Vec<u8>>>(vec: V) -> Self {
        let vec = vec.into();
        assert!(vec.len() >= ID_LEN);

        Self(
            vec.into_iter()
                .enumerate()
                .take(ID_LEN)
                .fold([0; ID_LEN], |mut buf, (i, u)| {
                    buf[i] = *u;
                    buf
                }),
        )
    }

    /// Create an identity from an exactly length-matched byte slice
    ///
    /// This function will panic, if the provided slice isn't exactly
    /// the length of the underlying identity implementation (see
    /// `ID_LEN`)
    pub fn from_bytes(buf: &[u8]) -> Self {
        assert_eq!(buf.len(), ID_LEN);
        Self(
            buf.into_iter()
                .enumerate()
                .fold([0; ID_LEN], |mut buf, (i, u)| {
                    buf[i] = *u;
                    buf
                }),
        )
    }

    pub fn from_string(s: &String) -> Self {
        let v: Vec<u8> = s
            .split("-")
            .map(|s| {
                hex::decode(s).expect(
                    "Don't call from_string() on input that was not serialised by to_string()!",
                )
            })
            .collect::<Vec<Vec<u8>>>()
            .into_iter()
            .flatten()
            .collect();
        Self::from_bytes(&v)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Create an identity using a digest function
    ///
    /// This allows you to pass arbitrary length data which will
    /// result in a precise ID length data output.  The hash function
    /// is the cryptographic [blake2] cipher, so it can be used to
    /// turn secrets into identity information.
    ///
    /// This function requires the `digest` feature.
    ///
    /// [blake2]: https://blake2.net/
    #[cfg(feature = "digest")]
    pub fn with_digest<'vec, V: Into<&'vec Vec<u8>>>(vec: V) -> Self {
        use blake2::{
            digest::{Input, VariableOutput},
            VarBlake2b,
        };

        let mut hasher = VarBlake2b::new(ID_LEN).unwrap();
        hasher.input(vec.into());
        Self::truncate(&hasher.vec_result())
    }

    /// Generate a new random Identity
    #[cfg(feature = "random")]
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut buf = [0; ID_LEN];
        rng.fill_bytes(&mut buf);
        Self(buf)
    }

    /// Returns an iterator over the bytes of the identity
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a u8> {
        self.0.iter()
    }
}

/// Implement RAW `From` binary array
impl From<[u8; ID_LEN]> for Identity {
    fn from(i: [u8; ID_LEN]) -> Self {
        Self(i)
    }
}

/// Implement RAW `From` binary (reference) array
impl From<&[u8; ID_LEN]> for Identity {
    fn from(i: &[u8; ID_LEN]) -> Self {
        Self(i.clone())
    }
}

/// Implement binary array `From` RAW
impl From<Identity> for [u8; ID_LEN] {
    fn from(i: Identity) -> Self {
        i.0
    }
}

/// Implement binary array `From` RAW reference
impl From<&Identity> for [u8; ID_LEN] {
    fn from(i: &Identity) -> Self {
        i.0.clone()
    }
}

/// Implement RAW identity to binary array reference
impl AsRef<[u8]> for Identity {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Iterator for iterating over `Identity`
pub struct Iter {
    index: usize,
    ident: Identity,
}

impl Iterator for Iter {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.ident.0.get(self.index).map(|byte| *byte);
        self.index += 1;
        ret
    }
}

impl IntoIterator for Identity {
    type Item = u8;
    type IntoIter = Iter;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            index: 0,
            ident: self,
        }
    }
}

impl Serialize for Identity {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if ser.is_human_readable() {
            ser.serialize_str(&self.to_string())
        } else {
            ser.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        struct IdentityVisitor;

        impl IdentityVisitor {
            fn from_str<E: Error>(v: &str) -> Result<Identity, E> {
                let v: Vec<u8> = v
                    .split("-")
                    .map(|s| hex::decode(s).map_err(|e| E::custom(e)))
                    // I don't like this way of propagating errors up but the alternative
                    // is a for loop which i also don't like
                    .collect::<Result<Vec<Vec<u8>>, E>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                Self::from_bytes(&v)
            }

            fn from_bytes<E: Error, V: AsRef<[u8]>>(v: V) -> Result<Identity, E> {
                let v = v.as_ref();
                if v.len() != ID_LEN {
                    return Err(E::custom(format!(
                        "Expected {} bytes, got {}",
                        ID_LEN,
                        v.len()
                    )));
                }

                Ok(Identity(v.iter().enumerate().take(ID_LEN).fold(
                    [0; ID_LEN],
                    |mut buf, (i, u)| {
                        buf[i] = *u;
                        buf
                    },
                )))
            }
        }

        impl<'de> Visitor<'de> for IdentityVisitor {
            type Value = Identity;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                write!(
                    f,
                    "Either a {l} byte array or a hex string representing {l} bytes",
                    l = ID_LEN
                )
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

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(IdentityVisitor)
        } else {
            deserializer.deserialize_bytes(IdentityVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bincode;
    use serde_json;

    #[test]
    #[cfg(not(features = "aligned"))]
    fn json_serde() {
        let s = b"Yes, we will make total destroy.";
        let i = Identity::truncate(&s.to_vec());
        let v = serde_json::to_string(&i).unwrap();
        assert_eq!(
            v,
            "\"5965-732C-2077-6520-7769-6C6C-206D-616B-6520-746F-7461-6C20-6465-7374-726F-792E\""
        );
        let i2 = serde_json::from_str(&v).unwrap();
        assert_eq!(i, i2);
    }

    #[test]
    #[cfg(not(features = "aligned"))]
    fn bincode_serde() {
        let s = b"Yes, we will make total destroy.";
        let i = Identity::truncate(&s.to_vec());
        let v: Vec<u8> = bincode::serialize(&i).unwrap();
        assert_eq!(
            v,
            vec![
                32, 0, 0, 0, 0, 0, 0, 0, 89, 101, 115, 44, 32, 119, 101, 32, 119, 105, 108, 108,
                32, 109, 97, 107, 101, 32, 116, 111, 116, 97, 108, 32, 100, 101, 115, 116, 114,
                111, 121, 46
            ],
        );
        let i2 = bincode::deserialize(&v).unwrap();
        assert_eq!(i, i2);
    }

    #[test]
    #[cfg(features = "aligned")]
    fn sized() {
        assert_eq!(crate::ID_LEN, size_of::<usize>());
    }

    /// This is the default length
    #[test]
    #[cfg(not(features = "aligned"))]
    fn sized() {
        assert_eq!(crate::ID_LEN, 32);
    }
}
