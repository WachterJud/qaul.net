//! A tcp overlay netmod to connect router across the internet

mod error;
mod peers;
mod proto;
mod socket;

pub use error::{Error, Result};

pub(crate) use peers::{Peers, Peer, LinkState};
pub(crate) use proto::Packet;
pub(crate) use socket::Socket;

use async_std::sync::{Arc, Receiver, RwLock};
use async_trait::async_trait;
use netmod::{self, Endpoint as EndpointExt, Frame, Target};
use std::net::SocketAddr;

/// Define the runtime mode for this endpount
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Static,
    Dynamic,
}

impl Mode {
    pub(crate) fn dynamic(&self) -> bool {
        self == &Mode::Dynamic
    }
}

#[derive(Clone)]
pub struct Endpoint {
    mode: Arc<RwLock<Mode>>,
    socket: Arc<Socket>,
    peers: Arc<Peers>,
    inbox: Option<Receiver<(Frame, usize)>>,
}

impl Endpoint {
    /// Create a new endpoint on an interface and port
    #[tracing::instrument(level = "info")]
    pub async fn new(addr: &str, port: u16, name: &str) -> Result<Self> {
        Ok(Self {
            mode: Arc::new(RwLock::new(Mode::Static)),
            socket: Socket::new(addr, port, name).await?,
            peers: Peers::new(),
            inbox: None,
        })
    }

    /// Set the runtime mode
    #[tracing::instrument(skip(self), level = "info")]
    pub async fn mode(&self, mode: Mode) {
        *self.mode.write().await = mode;
    }

    /// Load a set of peers, replacing the old peer list
    #[tracing::instrument(skip(self, peers), level = "info")]
    pub async fn load_peers<I: Into<SocketAddr>>(&self, peers: Vec<I>) -> Result<()> {
        self.peers.load(peers).await?;
        if let Some(_) = self.inbox {
            self.update_peers().await;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "info")]
    pub async fn start(&mut self) {
        self.inbox = Some(self.socket.start(*self.mode.read().await, &self.peers));
        self.update_peers().await;
    }

    async fn update_peers(&self) {
        let known = self.peers.all_known().await;
        for peer in known {
            if let Some(dst) = peer.dst {
                self.socket.introduce(peer.id, dst).await;
            }
        }
    }
}

#[async_trait]
impl EndpointExt for Endpoint {
    fn size_hint(&self) -> usize {
        0
    }

    #[tracing::instrument(skip(self, frame), level = "info")]
    async fn send(&self, frame: Frame, target: Target) -> netmod::Result<()> {
        match target {
            Target::Single(t) => {
                let addr = match self.peers.peer_with_id(t as usize).await {
                    Some(p) => match p.dst {
                        Some(a) => a,
                        None => { return Err(netmod::Error::ConnectionLost); }
                    },
                    None => { return Err(netmod::Error::ConnectionLost); }
                };
                self.socket.send(addr, frame).await.unwrap()
            },
            Target::Flood => self.socket.send_all(frame).await.unwrap(),
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "info")]
    async fn next(&self) -> netmod::Result<(Frame, Target)> {
        match self.inbox {
            Some(ref ib) => match ib.recv().await {
                Some((f, id)) => Ok((f, Target::Single(id as u16))),
                None => Err(netmod::Error::ConnectionLost),
            },
            None => Err(netmod::Error::ConnectionLost),
        }
    }
}

#[async_std::test]
async fn trivial() {
    use async_std::{future, task};
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        time::Duration,
    };

    let mut a = Endpoint::new("127.0.0.1", 10000, ">> A").await.unwrap();
    a.load_peers(vec![SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 11000)])
        .await
        .unwrap();

    let mut b = Endpoint::new("127.0.0.1", 11000, "> B").await.unwrap();
    b.load_peers(vec![SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 10000)])
        .await
        .unwrap();

    a.start().await;
    b.start().await;

    task::sleep(Duration::from_secs(1)).await;

    future::timeout(Duration::from_secs(5), async {
        let f = Frame::dummy();
        a.send(f.clone(), Target::Single(0)).await.unwrap();
        assert_eq!(b.next().await.unwrap().0, f);
    })
    .await
    .unwrap();
}

/// This test establishes a connection between two peers and then
/// let's them bounce keep-alive's back and forth for about 1 minute
/// to test stability.
///
/// This test should usually be ignored!
#[async_std::test]
#[ignore]
async fn akward_silence() {
    use async_std::task;
    use std::{
        net::{Ipv4Addr, SocketAddrV4},
        time::Duration,
    };

    println!("Starting two sockets to talk to each other now...");
    let mut a = Endpoint::new("127.0.0.1", 10000, ">> A").await.unwrap();
    let mut b = Endpoint::new("127.0.0.1", 11000, "> B").await.unwrap();

    a.load_peers(vec![SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 11000)])
        .await
        .unwrap();
    a.start().await;

    b.load_peers(vec![SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 10000)])
        .await
        .unwrap();
    b.start().await;

    task::sleep(Duration::from_secs(120)).await;
}
