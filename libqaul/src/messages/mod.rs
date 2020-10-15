//! Network message types and utilities

// Public exports
pub use crate::api::messages::{IdType, Message, Mode, MsgId, MsgQuery, MsgRef, SigTrust, ID_LEN};

mod store;
pub(crate) use self::store::{MsgStore, TAG_UNREAD};

#[cfg(feature = "generate-message")]
pub(crate) mod generator;

use crate::{
    error::{bincode, Result},
    helpers::Tag,
    security::Sec,
    users::UserStore,
};
use ratman::{
    netmod::Recipient as RatRecipient, Identity, Message as RatMessage, Router, TimePair,
};
use serde::{Deserialize, Serialize};

/// An internal wrapper around an incomplete Message
///
/// Because message signatures are computed based on the data provided
/// by the envelope, this means that changing the layout will **break
/// all message signature verification!** Keep that in mind!
///
/// Apart from that it contains all data that will be present in the
/// Message set generated for ratman routing.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub(crate) id: MsgId,
    pub(crate) sender: Identity,
    pub(crate) associator: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) tags: Vec<Tag>,
}

/// A `ratman::Message` set prototype structure
pub(crate) struct RatMessageProto {
    /// The high level `Message` to send and validate for
    pub(crate) env: Envelope,
    /// Readdressed `Recipient` information
    pub(crate) recipient: RatRecipient,
}

impl RatMessageProto {
    pub(crate) async fn build(&self, store: &UserStore) -> RatMessage {
        let sender = self.env.sender;
        let keypair = store.get_key(sender).await;

        // Serialise the envelope into a temporary payload.  The
        // envelope contains all data that is libqaul specific and
        // can't (or shouldn't) be taken from the ratman message
        // headers.
        let raw_payload = bincode::serialize(&self.env).unwrap();

        // Encrypt the payload only if the recipient is a single user
        let payload = match self.recipient {
            RatRecipient::User(id) => Sec::encrypt(keypair.clone(), id, &raw_payload),
            RatRecipient::Flood => raw_payload,
        };

        let sender = self.env.sender;
        let recipient = self.recipient;

        RatMessage {
            // Ratman generates a new message ID here to keep the real
            // message ID a secret and prevents header inspection to
            // figure out who is talking to whom.
            id: MsgId::random(),
            sender,
            recipient,
            payload,
            timesig: TimePair::sending(),

            // Payloads are encrypted and signed via libnacl, which
            // means that no payload can be forged. Ratman still takes
            // a signature argument at the moment for other
            // applications, and to keep the possibility to add
            // further verifications down the line.
            sign: vec![],
        }
    }
}

pub(crate) struct MsgUtils;

impl MsgUtils {
    /// Sends a `RatMessageProto`, calls a set of `send` commands
    pub(crate) async fn send(
        store: &UserStore,
        router: &Router,
        msg: RatMessageProto,
    ) -> Result<()> {
        Ok(router
            .send(msg.build(store).await)
            .await
            .map_err(|_| bincode())?)
    }

    pub(crate) fn extract_simple_payload(msg: &RatMessage) -> Option<Vec<u8>> {
        let RatMessage { payload, .. } = msg;
        let Envelope { payload, .. } = bincode::deserialize(&payload).ok()?;
        Some(payload)
    }

    /// Process incoming RATMAN message, verifying it's signature and payload
    pub(crate) async fn process(msg: RatMessage, store: &UserStore) -> Result<Message> {
        let RatMessage {
            id: _,
            sender,
            recipient,
            payload,
            timesig: _, // TODO: use!
            sign: _,
        } = msg;

        // Decrypt only if the message was directly addressed
        let payload = match recipient {
            RatRecipient::User(recp) => {
                let keypair = store.get_key(recp).await;

                // Decrypting the message makes sure the inner payload
                // structure was intact, as well as making sure the
                // signature is unharmed.  We want to drop a message
                // when the signature verification fails.
                Sec::decrypt(keypair, sender, &payload)?
            }
            RatRecipient::Flood => payload,
        };

        let Envelope {
            id,
            sender: _,
            associator,
            payload,
            tags,
        } = bincode::deserialize(&payload).unwrap();

        Ok(Message {
            id: id.into(),
            sender,
            associator,
            tags: tags.into(),
            payload,
        })
    }
}
