use super::Locked;
use crate::Message;

use async_notify::Notify;
use async_std::{
    future::{self, Future},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use netmod::{Frame, SeqId};
use std::collections::{BTreeMap, VecDeque};

/// Local frame collector state holder
#[derive(Default)]
pub(super) struct State {
    incoming: Notify<Locked<Notify<BTreeMap<SeqId, Notify<VecDeque<Frame>>>>>>,
    done: Locked<Notify<VecDeque<Message>>>,
}

impl State {
    /// Create a new state (oh no)
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Poll for completed messages from the outside world
    #[tracing::instrument(skip(self), level = "trace")]
    pub(super) async fn completed(&self) -> Message {
        let done = Arc::clone(&self.done);
        future::poll_fn(|ctx| {
            let lock = &mut done.lock();
            match unsafe { Pin::new_unchecked(lock).poll(ctx) } {
                Poll::Ready(ref mut not) => match not.pop_front() {
                    Some(f) => {
                        info!("Received new message for local service");
                        Poll::Ready(f)
                    }
                    None => {
                        if !Notify::has_waker(not) {
                            trace!("No new frames; registering waker");
                        }

                        Notify::register_waker(not, ctx.waker());
                        Poll::Pending
                    }
                },
                _ => Poll::Pending,
            }
        })
        .await
    }

    /// Poll for new work on a particular frame sequence
    pub(super) async fn get(&self, seq: &SeqId) -> Frame {
        let incoming = Arc::clone(&self.incoming);
        future::poll_fn(|ctx| {
            let lock = &mut incoming.lock();
            match unsafe { Pin::new_unchecked(lock).poll(ctx) } {
                Poll::Ready(ref mut map) => match map.get_mut(seq) {
                    Some(ref mut vec) if vec.len() > 0 => Poll::Ready(vec.pop_front().unwrap()),
                    Some(ref mut vec) => {
                        Notify::register_waker(vec, ctx.waker());
                        Poll::Pending
                    }
                    None => unimplemented!(), // No work queue _should_ never happen
                },
                _ => Poll::Pending,
            }
        })
        .await
    }

    /// Yield a finished message to the state
    #[tracing::instrument(skip(self), level = "trace")]
    pub(super) async fn finish(&self, msg: Message) {
        info!("Finishing up message collection");
        let mut done = self.done.lock().await;
        done.push_back(msg);
        Notify::wake(&mut *done);
    }

    /// Queue a new frame to the state
    pub(super) async fn queue(&self, seq: SeqId, frame: Frame) {
        let mut map = self.incoming.lock().await;
        let vec = map.entry(seq).or_default();
        vec.push_back(frame);
        Notify::wake(vec);
    }

    /// Get the current number of queued frames for diagnostic and testing
    #[cfg(test)]
    pub(crate) async fn num_queued(&self) -> usize {
        self.incoming
            .lock()
            .await
            .iter()
            .fold(0, |acc, (_, vec)| acc + vec.len())
    }

    /// Get the current number of completed messages
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) async fn num_completed(&self) -> usize {
        self.done.lock().await.len()
    }
}
