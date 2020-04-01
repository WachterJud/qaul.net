use super::*;
use libqaul::{
    api::SubId,
    contacts::ContactEntry,
    messages::{Message, MsgId, MsgRef},
    users::{UserAuth, UserProfile},
    Identity,
};

#[feature(chat)]
use qaul_chat::{
    room::{Room, RoomId},
    Chat, ChatMessage,
};

#[feature(voices)]
use qaul_voices::api::{CallId, IncomingCall, StreamMetadata, CallStatus};

use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

/// Represents a libqaul RPC request envelope
///
/// Because in some rpc systems requests will be processed in a
/// non-knowable order, making it hard to associtate requests with
/// responses.  This is what the request ID is for, and should be set,
/// even on systems that don't have this problem.
#[derive(Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub id: String,
    pub data: EnvelopeType,
}

/// A generic wrapper for requests and responses
///
/// In the rpc layer, the return data is then namespaced as "request"
/// and "response", which should be used to disambiguate data on the
/// wire.
#[derive(Clone, Serialize, Deserialize)]
pub enum EnvelopeType {
    /// A libqaul request
    Request(Request),
    /// A libqaul response
    Response(Response),
}

/// A wrapper enum to disambiguate request types in the envelope.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Request {
    /// Poll the next chat message
    #[cfg(feature = "chat")]
    ChatMsgNext(chat::messages::Next),

    /// Create a subscription for chat messages
    #[cfg(feature = "chat")]
    ChatMsgSub(chat::messages::Subscribe),

    /// Send a chat message
    #[cfg(feature = "chat")]
    ChatMsgSend(chat::messages::Send),

    /// Query the chat message store
    #[cfg(feature = "chat")]
    ChatMsgQuery,

    /// List all available chat rooms
    #[cfg(feature = "chat")]
    ChatRoomList(chat::rooms::List),

    /// Get data about a chat room
    #[cfg(feature = "chat")]
    ChatRoomGet(chat::rooms::Get),

    /// Create a new chat room
    #[cfg(feature = "chat")]
    ChatRoomCreate(chat::rooms::Create),

    /// Modify a chat room
    #[cfg(feature = "chat")]
    ChatRoomModify(chat::rooms::Modify),

    /// Delete a chat room
    #[cfg(feature = "chat")]
    ChatRoomDelete(chat::rooms::Delete),

    /// Modify a user's contact
    ContactModify(contacts::Modify),

    /// Get a user contact
    ContactGet(contacts::Get),

    /// Query a user's contacts
    ContactQuery(contacts::Query),

    /// Get all user contacts
    ContactAll(contacts::All),

    /// Send a raw libqaul message
    MsgSend(messages::Send),

    /// Poll the next raw libqaul message
    MsgNext(messages::Next),

    /// Create a subscription for raw libqaul messages
    MsgSub(messages::Subscribe),

    /// Query existing raw libqaul messages
    MsgQuery(messages::Query),

    /// List all available users
    UserList(users::List),

    /// List locally available users
    UserListLocal(users::ListLocal),

    /// List remote available users
    UserListRemote(users::ListRemote),

    /// Create a new user
    UserCreate(users::Create),

    /// Delete a local user
    UserDelete(users::Delete),

    /// Change a user's passphrase
    UserChangePw(users::ChangePw),

    /// Login as a user to get an auth token
    UserLogin(users::Login),

    /// End a user session
    UserLogout(users::Logout),

    /// Get data on a particular user
    UserGet(users::Get),

    /// Update a user
    UserUpdate(users::Update),

    #[cfg(feature = "voices")]
    /// Initiate a call to a remote user
    VoicesMakeCall(voices::MakeCall),

    #[cfg(feature = "voices")]
    /// Accept a call from a remote user
    VoicesAcceptCall(voices::AcceptCall),

    #[cfg(feature = "voices")]
    /// Reject a call
    VoicesRejectCall(voices::RejectCall),

    #[cfg(feature = "voices")]
    /// Terminate a call
    VoicesHangUp(voices::HangUp),

    #[cfg(feature = "voices")]
    /// Wait for the next incoming call
    VoicesNextIncoming(voices::NextIncoming),

    #[cfg(feature = "voices")]
    /// Get the stream metadata for the remote end of a call
    VoicesGetMetadata(voices::GetMetadata),

    #[cfg(feature = "voices")]
    /// Push voice samples on to the outgoing call buffer
    VoicesPushVoice(voices::PushVoice),

    #[cfg(feature = "voices")]
    /// Get the status of a call
    VoicesGetStatus(voices::GetStatus),

    #[cfg(feature = "voices")]
    /// Subscribe to the incoming voice samples of a call
    VoicesNextVoice(voices::NextVoice),

    #[cfg(feature = "voices")]
    /// Await the termination of a call
    VoicesOnHangup(voices::OnHangup),
}

/// Wrap around all possible response values for piped Rpc protocols
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    /// Return an auth object
    Auth(UserAuth),

    /// Return a set of chat messages
    #[cfg(feature = "chat")]
    ChatMessage(Vec<ChatMessage>),

    /// Return a set of contact entries
    Contact(Vec<ContactEntry>),

    /// Return an error type
    Error(String),

    /// Return a set of message
    Message(Vec<Message>),

    /// Return a message ID
    MsgId(MsgId),

    /// Return chat room data
    #[cfg(feature = "chat")]
    Room(Room),

    /// Get a set of chat room IDs
    #[cfg(feature = "chat")]
    RoomId(Vec<RoomId>),

    /// Confirmation for a new subscription
    Subscription(SubId),

    /// A generic success message
    Success,

    /// Return a set of user profiles
    User(Vec<UserProfile>),

    /// Return available user IDs
    UserId(Vec<Identity>),

    /// A call id
    #[cfg(feature = "voices")]
    CallId(CallId),

    /// An incoming call
    #[cfg(feature = "voices")]
    IncomingCall(IncomingCall),

    /// Metadata about a voice stream 
    #[cfg(feature = "voices")]
    StreamMetadata(StreamMetadata),

    /// The status of a call 
    #[cfg(feature = "voices")]
    CallStatus(CallStatus),

    /// A set of voice samples
    #[cfg(feature = "voices")]
    VoiceData(Vec<i16>),
}

impl From<UserAuth> for Response {
    fn from(auth: UserAuth) -> Self {
        Response::Auth(auth)
    }
}

#[cfg(feature = "chat")]
impl From<ChatMessage> for Response {
    fn from(msg: ChatMessage) -> Self {
        Response::ChatMessage(vec![msg])
    }
}

#[cfg(feature = "chat")]
impl From<Vec<ChatMessage>> for Response {
    fn from(msgs: Vec<ChatMessage>) -> Self {
        Response::ChatMessage(msgs)
    }
}

impl From<ContactEntry> for Response {
    fn from(contact: ContactEntry) -> Self {
        Response::Contact(vec![contact])
    }
}

impl From<Vec<ContactEntry>> for Response {
    fn from(contacts: Vec<ContactEntry>) -> Self {
        Response::Contact(contacts)
    }
}

impl<T: Into<Response>, E: Display> From<Result<T, E>> for Response {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(t) => t.into(),
            Err(e) => Response::Error(e.to_string()),
        }
    }
}

impl From<MsgRef> for Response {
    fn from(msg: MsgRef) -> Self {
        Response::Message(vec![msg.as_ref().clone()])
    }
}

impl From<Vec<MsgRef>> for Response {
    fn from(msgs: Vec<MsgRef>) -> Self {
        Response::Message(msgs.into_iter().map(|msg| msg.as_ref().clone()).collect())
    }
}

#[cfg(feature = "chat")]
impl From<Room> for Response {
    fn from(room: Room) -> Self {
        Response::Room(room)
    }
}

impl From<()> for Response {
    fn from(_: ()) -> Self {
        Self::Success
    }
}

impl From<UserProfile> for Response {
    fn from(user: UserProfile) -> Self {
        Response::User(vec![user])
    }
}

impl From<Vec<UserProfile>> for Response {
    fn from(users: Vec<UserProfile>) -> Self {
        Response::User(users)
    }
}

#[cfg(feature = "voices")]
impl From<IncomingCall> for Response {
    fn from(incoming: IncomingCall) -> Self {
        Response::IncomingCall(incoming)
    }
}

#[cfg(feature = "voices")]
impl From<StreamMetadata> for Response {
    fn from(metadata: StreamMetadata) -> Self {
        Response::StreamMetadata(metadata)
    }
}

#[cfg(feature = "voices")]
impl From<CallStatus> for Response {
    fn from(status: CallStatus) -> Self {
        Response::CallStatus(status)
    }
}
