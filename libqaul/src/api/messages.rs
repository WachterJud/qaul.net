use crate::{
    error::{Error, Result},
    helpers::{QueryResult, Subscription, Tag, TagSet},
    messages::{Envelope, MsgUtils, RatMessageProto, TAG_UNREAD},
    qaul::{Identity, Qaul},
    services::Service,
    users::UserAuth,
};

use ratman::netmod::Recipient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use libqaul_types::messages::{IdType, Message, Mode, MsgId, MsgQuery, MsgRef, SigTrust, ID_LEN};

fn mode_to_recp(sm: Mode) -> Recipient {
    match sm {
        Mode::Flood => Recipient::Flood,
        Mode::Std(id) => Recipient::User(id),
    }
}

/// Interface to access messages from the network
pub struct Messages<'chain> {
    pub(crate) q: &'chain crate::Qaul,
}

impl<'qaul> Messages<'qaul> {
    /// Drop this scope and return back to global `Qaul` scope
    pub fn drop(&'qaul self) -> &'qaul Qaul {
        self.q
    }

    /// Send a message with arbitrary payload into the network
    ///
    /// Because the term `Message` is overloaded slightly in
    /// `libqaul`, here is a small breakdown of what a message means
    /// in this context.
    ///
    /// The Service API provides an interface to communicate with
    /// other users on a qaul network. These messages are relatively
    /// low-level, meaning that their payload (for example) is simply
    /// a `Vec`, and it's left to a service to do anything meaningful
    /// with it.
    ///
    /// However when users write text-messages to each other in
    /// qaul.net, these are being sent via the `messaging` service,
    /// which implements it's own `Message`, on top of `libqaul`. In
    /// that case a message is plain text and can have binary
    /// attachments.
    ///
    /// Underlying `libqaul`, the routing layer (`RATMAN`) uses the
    /// term Message to refer to the same concept as a Service API
    /// message, with some more raw data inlined, such as signatures
    /// and checksums. Fundamentally they share the same idea of what
    /// a payload or recipient is however, and payloads that are
    /// unsecured in a Service API message will have been encrypted by
    /// the time that `RATMAN` handles them.
    pub async fn send<S, T>(
        &self,
        user: UserAuth,
        mode: Mode,
        id_type: IdType,
        service: S,
        tags: T,
        payload: Vec<u8>,
    ) -> Result<MsgId>
    where
        S: Into<String>,
        T: Into<TagSet>,
    {
        let (sender, _) = self.q.auth.trusted(user)?;
        let recipient = mode_to_recp(mode);
        let associator = service.into();
        let id = id_type.consume();
        let tags: TagSet = tags.into();
        println!("Sending `{}` with tags {:?}", id, tags);

        let env = Envelope {
            id,
            sender,
            associator: associator.clone(),
            payload: payload.clone(),
            tags: tags.iter().cloned().collect(),
        };

        println!("Sending message with ID `{:?}`", id);
        println!("Sending message to {:?}", recipient);

        // Only insert the message into the store if the Id is unique!
        if !self.q.messages.probe_id(sender, id).await {
            self.q
                .messages
                .insert_local(
                    sender,
                    Arc::new(Message {
                        id,
                        sender,
                        associator,
                        tags,
                        payload,
                    }),
                    mode,
                )
                .await;

            assert!(self.q.messages.probe_id(sender, id).await);
        }

        MsgUtils::send(
            &self.q.users,
            &self.q.router,
            RatMessageProto { env, recipient },
        )
        .await
        .map(|_| id)
    }

    /// Subscribe to a stream of future message updates
    pub async fn subscribe<S, T>(
        &self,
        user: UserAuth,
        service: S,
        tags: T,
    ) -> Result<Subscription<Message>>
    where
        S: Into<Service>,
        T: Into<TagSet>,
    {
        let (id, _) = self.q.auth.trusted(user)?;
        Ok(self
            .q
            .messages
            .subscribe(id, service.into(), tags.into())
            .await)
    }

    /// Query for messages in the store, according to some parameters
    ///
    /// A query is always user authenticated, and normally associated
    /// to a service, but it doesn't have to be, if `god-mode` is
    /// enabled in the libqaul instance.
    ///
    /// The query parameters can be specified via the [`Query`]
    /// builder type which allows for very selective constraints.  The
    /// return of this function is a Wrapper around a result iterator
    /// that can return batches, or skip items dynamically.
    pub async fn query(
        &self,
        user: UserAuth,
        service: impl Into<Service>,
        query: MsgQuery,
    ) -> Result<QueryResult<Message>> {
        let (id, _) = self.q.auth.trusted(user)?;
        Ok(self.q.messages.query(id, service.into(), query).await)
    }
}
