//! New peer abstraction module
//!
//! The lifecycle of a peer is encoded is the following flow-chart:
//!
//! ```text
//!                               +-> Failed to -+
//!                               |    connect   |
//!                               |              v
//! Start --- for all peers ---> Start a connection
//!                                |            |
//!                                |            |
//!                                v            v
//!                       REVERSE stream       Send a HELLO to init
//!                      (already) exists       reverse connection
//!                               |                      |
//!                               |                      |
//!                               |                      v
//!                               v               Wait for REVERSE connection
//!                      Valid DUPLEX                   |
//!                       connection  <-----------------+
//! ```
//!
//! If at any point sending a message fails, this re-connection needs
//! to be repeated and the packet held until then.
//!
//! All operations on a peer are async, and will be queued via a
//! channel, which means they will return immediately, even if the
//! connection is currently down.

use crate::{AtomPtr, IoPair, LinkType, LockedStream, Packet, PacketBuilder};
use async_std::{
    future::timeout,
    io::prelude::WriteExt,
    net::TcpStream,
    sync::{Arc, RwLock},
    task,
};
use bincode::serialize;
use byteorder::{BigEndian, ByteOrder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{net::SocketAddr, time::Duration};
use tracing::{error, trace};

/// Utility module to generate monotonic peer IDs
mod id {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ID_CTR: AtomicUsize = AtomicUsize::new(0);

    /// Get the next monotonically increasing ID
    pub fn next() -> usize {
        ID_CTR.fetch_add(1, Ordering::Relaxed)
    }
}

/// Address from which packets are sent
pub(crate) type SourceAddr = SocketAddr;

/// Address to which packets are sent
pub(crate) type DstAddr = SocketAddr;

/// Encode the different states a `Peer` can be in
#[derive(Debug)]
pub(crate) enum PeerState {
    /// Only a receiving channel exists
    ///
    /// This is either the case for unknown dynamic peers, or a
    /// race-condition on static peers.
    RxOnly,
    /// Only a transmission channel exists
    ///
    /// This is the inverse of RxOnly, and usually a race-condition,
    /// unless the local node is running in DYNAMIC mode
    TxOnly,
    /// A valid two-way connection
    Duplex,
    /// Something has gone really wrong
    Invalid,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Peer {
    /// Unique numeric Id for each peer
    pub(crate) id: usize,
    /// Peer source address
    src: AtomPtr<Option<SourceAddr>>,
    /// Peer destination address
    dst: Option<DstAddr>,
    /// Sending stream for this peer (if it existst)
    sender: AtomPtr<LockedStream>,
    /// The type of link this maintains
    _type: LinkType,
    /// Secret run condition
    #[doc(hidden)]
    _run: Arc<AtomicBool>,
    /// Store packets until they can be delivered
    io: Arc<IoPair<Packet>>,
}

impl Peer {
    /// Initialise a new peer only with it's source address
    ///
    /// This function is meant to initialise a peer who has introduced
    /// itself to this node, but where an outgoing connection couldn't
    /// be established yet.  This indicates either a race condition
    /// which will be resolved soon (because the local peer
    /// initialisation loop hasn't spawned the sending channel yet),
    /// or an unknown peer when running in `dynamic` mode
    pub(crate) fn from_src(src: SourceAddr) -> Arc<Self> {
        Arc::new(Self {
            id: id::next(),
            src: AtomPtr::new(Some(src)),
            _run: Arc::new(true.into()),
            ..Default::default()
        })
    }

    /// Open a connection to this peer
    ///
    /// While this function returns immediately, it spawns an async
    /// worker that will try to establish a connection to the peer,
    /// exiting until `stop()` is called on this peer
    #[tracing::instrument(level = "trace")]
    pub(crate) fn open(dst: DstAddr, port: u16, _type: LinkType) -> Arc<Self> {
        let p = Arc::new(Self {
            id: id::next(),
            dst: Some(dst),
            _run: Arc::new(true.into()),
            _type,
            ..Default::default()
        });

        // Start sender loop and send a hello
        Arc::clone(&p).run_io_sender(port, _type);
        task::block_on(async { Arc::clone(&p).send(Packet::Hello { port, _type }).await });

        return p;
    }

    /// Set this peer's source address
    pub(crate) fn set_src<O: Into<Option<SourceAddr>>>(&self, src: O) {
        self.src.swap(src.into());
    }

    pub(crate) async fn set_stream(&self, s: LockedStream) {
        self.sender.swap(s);
    }

    /// Stop all tasks associated with this peer
    pub(crate) fn stop(&self) {
        self._run.fetch_and(false, Ordering::Relaxed);
    }

    /// Get the current state for this peer
    pub(crate) fn state(&self) -> PeerState {
        match (self.get_src(), self.dst) {
            (Some(_), Some(_)) => PeerState::Duplex,
            (Some(_), None) => PeerState::RxOnly,
            (None, Some(_)) => PeerState::TxOnly,
            (None, None) => unreachable!(),
        }
    }

    /// Get the type for this link
    pub(crate) fn link_type(&self) -> LinkType {
        self._type
    }

    /// Internal utility to verify that this peer is still alive
    pub(crate) fn alive(&self) -> bool {
        self._run.load(Ordering::Relaxed)
    }

    /// Call for each packet in the output stream
    async fn send_packet(self: &Arc<Self>, p: &Packet) -> Option<()> {
        let r = self.sender.get_ref();
        let mut s = r.write().await;
        match *s {
            Some(ref mut stream) => {
                let addr = match stream.peer_addr() {
                    Ok(addr) => addr.to_string(),
                    Err(_) => {
                        std::mem::swap(&mut *s, &mut None);
                        return None;
                    }
                };

                // And woosh!
                let buf = p.serialize();
                if let Err(e) = stream.write_all(&buf).await {
                    error!("Failed to send message: {}!", e.to_string());

                    // We mark ourselves as missing uplink
                    std::mem::swap(&mut *s, &mut None);

                    return None;
                }

                match p {
                    Packet::Hello { .. } => {
                        trace!("Sending HELLO to {}", addr);

                        if self._type == LinkType::Bidirect {
                            let _self = Arc::clone(self);
                            task::spawn(_self.wait_for_ack());
                        }
                    }
                    _ => {}
                }

                Some(())
            }
            None => unreachable!(),
        }
    }

    /// Run a listener to wait for an ACK to be returned to this connection
    async fn wait_for_ack(self: Arc<Self>) {
        let t = timeout(Duration::from_secs(10), async {
            loop {
                let r = self.sender.get_ref();
                let mut s = r.write().await;
                if s.is_none() {
                    break;
                }

                if timeout(Duration::from_millis(1), async {
                    let mut pb = PacketBuilder::new((*s).as_mut().unwrap());
                    match pb.parse().await {
                        Ok(_) => match pb.build() {
                            Some(Packet::Ack) => trace!("Received an ACK."),
                            _ => error!("Invalid data (only ACKs)!"),
                        },
                        _ => {
                            std::mem::swap(&mut *s, &mut None);
                            error!("Failed to read ACK from sender stream");
                        }
                    }
                })
                .await
                .is_ok()
                {
                    break;
                }

                drop(s);
                task::sleep(Duration::from_millis(50)).await;
            }
        });

        // If the top-level timeout is ever hit...
        match t.await {
            Ok(_) => {}
            Err(_) => {
                // Remove the stream because it's probably dead
                let _ref = self.sender.get_ref();
                let mut s = _ref.write().await;
                std::mem::swap(&mut *s, &mut None);
            }
        }
    }

    /// This function will try sending a packet, initialising the
    /// output stream if it doesn't yet exist
    async fn send_or_introduce(self: &Arc<Self>, p: Packet, port: u16, _type: LinkType) {
        loop {
            if { self.sender.get_ref().read().await.is_some() } {
                // Send the packet and re-run the loop if we failed to send
                match self.send_packet(&p).await {
                    Some(_) => break,
                    None => continue, // send_packet sets sender = None if failed
                }
            } else {
                if _type == LinkType::Bidirect {
                    trace!("Sender is None, opening a connection first...");
                    Arc::clone(&self).introduce_blocking(port).await;
                }
            }
        }
    }

    /// Start an async worker to send packets to this peer
    ///
    /// The worker can be stopped after spawning by calling `stop()`.
    /// If at any time sending wasn't successful, this loop will
    /// automatically re-init the connection.
    ///
    /// There's currently no way to get diagnostics from failed sends
    /// back to ratman.  **FIXME**: implement this!
    pub(crate) fn run_io_sender(self: Arc<Self>, port: u16, _type: LinkType) {
        trace!("Running IO sender");
        task::spawn(async move {
            while let Some(p) = self.io.rx.recv().await {
                trace!("Queued packet {:?}", p);
                self.send_or_introduce(p, port, _type).await;

                if !self.alive() {
                    break;
                }
            }

            trace!("Shutting down packet sender for peer {}", self.id);
        });
    }

    /// Loop on a connection until it could be established!
    async fn introduce_blocking(self: Arc<Self>, port: u16) {
        let id = self.id.clone();
        let dst = self.dst.clone().unwrap();

        let run = Arc::clone(&self._run);
        let sender = Arc::clone(&self.sender.get_ref());
        let mut ctr = 0;

        while run.load(Ordering::Relaxed) {
            let pre = match ctr {
                0 => "".into(),
                n => format!("[retry #{}]", n),
            };

            // Exit if we are already connected
            if sender.read().await.is_some() {
                trace!(
                    "Peer `{}` (ID: {}) is already connected!",
                    dst.to_string(),
                    id
                );
                break;
            }

            trace!(
                "{}: Attempting to connect to peer `{}`",
                pre,
                dst.to_string()
            );
            let s = match TcpStream::connect(dst).await {
                Ok(s) => s,
                Err(_) => {
                    error!(
                        "Failed to connect to peer `{}`.  Starting timeout...",
                        dst.to_string()
                    );

                    // FIXME: Make this configurable
                    task::sleep(Duration::from_secs(5)).await;
                    ctr += 1;
                    continue;
                }
            };

            s.set_nodelay(true);

            trace!("Successfully connected to peer `{}`", &dst);
            let mut sender = sender.write().await;
            *sender = Some(s);
            break;
        }
    }

    /// Send some arbitrary packet to this peer
    ///
    /// If the connection has become invalid in the meantime, this
    /// function will automatically call introduce and wait for a
    /// connection to occur before retrying.  In this case it will
    /// spawn an async worker and return, even if the data was not
    /// successfully delivered.
    pub(crate) async fn send(&self, packet: Packet) {
        self.io.tx.send(packet).await;
    }

    pub(crate) fn get_src(&self) -> Option<SourceAddr> {
        *self.src.get_ref().clone()
    }

    pub(crate) fn get_dst(&self) -> Option<DstAddr> {
        self.dst.clone()
    }
}
