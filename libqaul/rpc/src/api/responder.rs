use crate::{api::Subscriber, QaulExt, QaulRpc, Request, Response, StreamResponder, Streamer};
use async_std::sync::Arc;
use libqaul::Qaul;

#[cfg(feature = "chat")]
use crate::{ChatExt, ChatRpc};
#[cfg(feature = "chat")]
use qaul_chat::Chat;

#[cfg(feature = "voice")]
use crate::{VoiceExt, VoiceRpc};
#[cfg(feature = "voice")]
use qaul_voice::Voice;

/// A type mapper to map RPC requests to libqaul and services
pub struct Responder<K: StreamResponder + Send + Sync + 'static> {
    /// Keeps track of subscription runs
    pub streamer: Arc<Streamer<K>>,

    pub qaul: Arc<Qaul>,

    #[cfg(feature = "chat")]
    pub chat: Arc<Chat>,

    #[cfg(feature = "voice")]
    pub voice: Arc<Voice>,
}

impl<K: StreamResponder + Send + Sync + 'static> Responder<K> {
    async fn respond_qaul<R, T>(&self, request: R) -> T
    where
        R: QaulRpc<Response = T> + Send + Sync,
        T: Send + Sync,
    {
        self.qaul.apply(request).await
    }

    #[cfg(feature = "chat")]
    async fn respond_chat<R, T>(&self, request: R) -> T
    where
        R: ChatRpc<Response = T> + Send + Sync,
        T: Send + Sync,
    {
        (&self.chat).apply(request).await
    }

    #[cfg(feature = "voice")]
    async fn respond_voice<R, T>(&self, request: R) -> T
    where
        R: VoiceRpc<Response = T> + Send + Sync,
        T: Send + Sync,
    {
        (&self.voice).apply(request).await
    }

    /// Primary responder matcher
    ///
    /// Takes in a request, calls the appropriate submodule/service,
    /// and then returns the result to the caller.
    ///
    // With this function we avoid having 100 functions that all do
    // basically the same thing, and the switching logic stays in one
    // place.  If you need some context as to why this is required,
    // what it does, and how it actually works, there's an RPC chapter
    // in the qaul.net manual for you to read up on these things.
    // If you have more questions afterwards, feel free to reach out
    // to us.  When touching this function, try to leave comments for
    // anything that's not immediately obvious.  Also: =^-^=!
    pub async fn respond(&self, req: Request) -> Response {
        // TODO: currently the ids all map into Response::UserId which is wrong
        match req {
            // =^-^= Generic RPC commands =^-^=
            Request::CancelSub(r) => match self.respond_qaul(r).await {
                Ok(id) => {
                    self.streamer.stop(id).await;
                    Response::Success
                }
                Err(e) => Response::Error(e.to_string()),
            },

            // =^-^= Chat Messages =^-^=
            #[cfg(feature = "chat")]
            Request::ChatMsgCreate(r) => self.respond_chat(r).await.into(),
            #[cfg(feature = "chat")]
            Request::ChatMsgSub(r) => self
                .respond_chat(r)
                .await
                .map(|sub| Response::Subscription(self.streamer.start(sub)))
                .unwrap_or_else(|e| Response::Error(e.to_string())),

            // =^-^= Chat Rooms =^-^=
            #[cfg(feature = "chat")]
            Request::ChatRoomList(r) => self.respond_chat(r).await.into(),
            #[cfg(feature = "chat")]
            Request::ChatRoomGet(r) => self.respond_chat(r).await.into(),
            #[cfg(feature = "chat")]
            Request::ChatLoadRoom(r) => self.respond_chat(r).await.into(),
            #[cfg(feature = "chat")]
            Request::ChatRoomCreate(r) => self.respond_chat(r).await.into(),
            #[cfg(feature = "chat")]
            Request::ChatRoomModify(r) => self.respond_chat(r).await.into(),

            // =^-^= Contacts =^-^=
            // FIXME: this should be a contacts type!
            Request::UserListRemote(r) => self.respond_qaul(r).await.into(),
            Request::ContactGet(r) => self.respond_qaul(r).await.into(),

            // TODO: Currently the "query" functions don't return
            // actual data, but just the IDs.  Maybe we should change
            // that in libqaul, but until then this RPC layer should
            // just mirror the base API.
            //
            // The usage here should probably be made nicer with a
            // From<Result<T, E>>, which is already implemented, but I
            // think we need to turbo-fish it somehow.  Anyway, future
            // me's problem :)
            Request::ContactQuery(r) => self
                .respond_qaul(r)
                .await
                .map(Into::into)
                .unwrap_or_else(|e| Response::Error(e.to_string())),
            Request::ContactAll(r) => self
                .respond_qaul(r)
                .await
                .map(Into::into)
                .unwrap_or_else(|e| Response::Error(e.to_string())),

            // =^-^= Messages =^-^=
            Request::MsgSend(r) => match self.respond_qaul(r).await {
                Ok(id) => Response::MsgId(id),
                Err(e) => Response::Error(e.to_string()),
            },

            // =^-^= Users =^-^=
            Request::UserList(r) => self.respond_qaul(r).await.into(),
            Request::UserIsAuthenticated(r) => match self.respond_qaul(r).await {
                Ok(()) => Response::Success,
                Err(_) => Response::Error("Not authorised".into()),
            },
            Request::UserCreate(r) => self.respond_qaul(r).await.into(),
            Request::UserDelete(r) => self.respond_qaul(r).await.into(),
            Request::UserChangePw(r) => self.respond_qaul(r).await.into(),
            Request::UserLogin(r) => self.respond_qaul(r).await.into(),
            Request::UserLogout(r) => self.respond_qaul(r).await.into(),
            Request::UserGet(r) => self.respond_qaul(r).await.into(),
            Request::UserUpdate(r) => self.respond_qaul(r).await.into(),

            // =^-^= Voices =^-^=
            #[cfg(feature = "voice")]
            Request::VoiceStartCall(r) => self
                .respond_voice(r)
                .await
                .map(|call_id| Response::CallId(call_id))
                .unwrap_or_else(|e| Response::Error(e.to_string())),
            #[cfg(feature = "voice")]
            Request::VoiceGetCalls(r) => self.respond_voice(r).await.into(),
            #[cfg(feature = "voice")]
            Request::VoiceGetCall(r) => self.respond_voice(r).await.into(),
            #[cfg(feature = "voice")]
            Request::VoiceInviteToCall(r) => self.respond_voice(r).await.into(),
            #[cfg(feature = "voice")]
            Request::VoiceJoinCall(r) => self.respond_voice(r).await.into(),
            #[cfg(feature = "voice")]
            Request::VoiceLeaveCall(r) => self.respond_voice(r).await.into(),
            #[cfg(feature = "voice")]
            Request::VoiceSubscribeInvites(r) => self
                .respond_voice(r)
                .await
                .map(|sub| Response::Subscription(self.streamer.start(sub)))
                .unwrap_or_else(|e| Response::Error(e.to_string())),
            #[cfg(feature = "voice")]
            Request::VoiceSubscribeCallEvents(r) => self
                .respond_voice(r)
                .await
                .map(|sub| Response::Subscription(self.streamer.start(sub)))
                .unwrap_or_else(|e| Response::Error(e.to_string())),

            tt => panic!(
                "Encountered unimplemented parse type: {:#?}\n...so sorry",
                tt
            ),
        }
    }
}
