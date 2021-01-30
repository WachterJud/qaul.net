use async_trait::async_trait;
use libqaul_rpc1::{Response, StreamResponder, Streamer};
use std::sync::Arc;
use tracing::warn;

pub(crate) fn setup_streamer() -> Arc<Streamer<StreamResp>> {
    Streamer::new(StreamResp {})
}

pub struct StreamResp {}

#[async_trait]
impl StreamResponder for StreamResp {
    async fn respond(self: Arc<Self>, r: Response) {
        warn!("Stream update unimplemented!");
    }
}
