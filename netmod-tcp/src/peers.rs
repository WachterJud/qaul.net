//! Peer tracking

use crate::error::PeerErrs;
use async_std::sync::{Arc, RwLock};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

type SourceAddr = SocketAddr;
type DstAddr = SocketAddr;

#[derive(Clone, Debug)]
struct Peer {
    src: Option<SocketAddr>,
    dst: SocketAddr,
    verified: bool,
}

/// Map the source port to the destination port of a target
type AddrMap = HashMap<SourceAddr, Peer>;

/// Maps the interface specific target id to the peers listening port
type IdMap = BTreeMap<usize, Peer>;

/// For quick queries if a peer is valid
type PeerMap = HashMap<DstAddr, usize>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum PeerState {
    /// We're friends actually
    Valid,
    /// New connection, but we know the IP
    Unverified,
    /// Unknown IP, ignore
    Unknown,
}

#[derive(Default)]
pub(crate) struct PeerList {
    addr_map: RwLock<AddrMap>,
    id_map: RwLock<IdMap>,
    peers: RwLock<PeerMap>,
    curr: RwLock<usize>,
}

impl PeerList {
    /// Create a new empty peer list
    pub(crate) fn new() -> Arc<Self> {
        Default::default()
    }

    pub(crate) async fn all_known(self: &Arc<Self>) -> Vec<(usize, SocketAddr)> {
        self.id_map
            .read()
            .await
            .iter()
            .map(|(id, peer)| (*id, peer.dst.clone()))
            .collect()
    }

    /// Get the state of a peer (unknown, unverified, or valid)
    pub(crate) async fn peer_state(self: &Arc<Self>, src: &SourceAddr) -> PeerState {
        let peers = self.peers.read().await;
        let id_map = self.id_map.read().await;

        use PeerState::*;
        peers.iter().fold(Unknown, |prev, (src_, id)| {
            let verified = match id_map.get(id) {
                Some(peer) => peer.verified,
                None => false
            };
            match prev {
                Unknown if src_.ip() == src.ip() && verified => Valid,
                Unknown if src_.ip() == src.ip() => Unverified,
                found => found,
            }
        })
    }

    /// Get the ID of a peer with the source socket address
    pub(crate) async fn get_id_by_src(self: &Arc<Self>, src: &SourceAddr) -> Option<usize> {
        self.addr_map
            .read()
            .await
            .get(src)
            .and_then(|Peer { dst, .. }| {
                async_std::task::block_on(async { self.get_id_by_dst(dst).await })
            })
    }

    /// Get the ID of a peer with the source socket address
    pub(crate) async fn get_id_by_dst(self: &Arc<Self>, src: &DstAddr) -> Option<usize> {
        self.peers.read().await.get(src).map(|id| *id)
    }

    /// Get the destination address based on the source address
    pub(crate) async fn get_dst_by_src(self: &Arc<Self>, src: &SourceAddr) -> Option<DstAddr> {
        self.addr_map
            .read()
            .await
            .get(src)
            .map(|peer| peer.dst.clone())
    }

    /// Add the source part of a peer based on the ip and dst port
    pub(crate) async fn add_src(
        self: &Arc<Self>,
        src: &SourceAddr,
        dst_port: u16,
    ) -> Option<usize> {
        let dst = {
            let mut s = src.clone();
            s.set_port(dst_port);
            s
        };

        // Lock all data stores
        let mut addr_map = self.addr_map.write().await;
        let mut id_map = self.id_map.write().await;
        let peers = self.peers.write().await;
        let mut curr = self.curr.write().await;

        // Return None if the peer is not known
        if peers.get(&dst).is_none() {
            return None;
        }

        // Either get a pre-assigned ID, or create a new one
        let id = match peers.get(&dst) {
            Some(id) => *id,
            None => {
                *curr += 1;
                *curr
            }
        };

        // Either get an existing peer, or create a new one
        let peer = match id_map.get_mut(&id) {
            Some(peer) => {
                peer.src = Some(src.clone());
                peer.verified = true;
                peer.clone()
            },
            None => Peer {
                src: Some(src.clone()),
                dst: dst.clone(),
                verified: true
            }
        };

        addr_map.insert(src.clone(), peer.clone());
        Some(id)
    }

    /// Remove the peer with the given ID, or do nothing if no such peer exists.
    pub(crate) async fn del_peer(
        self: Arc<Self>,
        id: usize 
    ) {
        // Lock all data stores
        let mut addr_map = self.addr_map.write().await;
        let mut id_map = self.id_map.write().await;

        if let Some(peer) = id_map.get(&id) {
            if let Some(src_addr) = peer.src {
                addr_map.remove (&src_addr);
            }
        }

        id_map.remove(&id);
    }

    pub(crate) async fn load<I: Into<SocketAddr>>(
        self: &Arc<Self>,
        peers: Vec<I>,
    ) -> Result<(), PeerErrs> {
        let new_peers: Vec<_> = peers.into_iter().map(Into::into).collect();

        let mut id_map = self.id_map.write().await;
        let mut peers = self.peers.write().await;
        let mut curr = self.curr.write().await;

        new_peers.into_iter().fold(Ok(()), |prev, addr| match prev {
            Ok(_) if peers.contains_key(&addr) => PeerErrs::new(addr),
            Err(e) if peers.contains_key(&addr) => Err(e.append(addr)),
            Ok(()) => {
                peers.insert(addr.clone(), *curr);
                let peer = Peer {
                    src: None,
                    dst: addr,
                    verified: false,
                };
                id_map.insert(*curr, peer.clone());

                *curr += 1;
                Ok(())
            }
            err @ Err(_) => {
                peers.insert(addr.clone(), *curr);
                let peer = Peer {
                    src: None,
                    dst: addr,
                    verified: false,
                };
                id_map.insert(*curr, peer);

                *curr += 1;
                err
            }
        })
    }

    /// Get the destination address based on the source address
    #[cfg(test)]
    pub(crate) async fn get_dst_by_id(self: &Arc<Self>, id: usize) -> Option<DstAddr> {
        self.id_map
            .read()
            .await
            .get(&id)
            .map(|peer| peer.dst.clone())
    }
}

#[async_std::test]
async fn load_peers() {
    use std::net::{Ipv4Addr, SocketAddrV4};

    let a1 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8000);
    let a2 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9000);

    let peers = PeerList::new();
    peers.load(vec![a1.clone(), a2.clone()]).await.unwrap();

    let id = peers.get_id_by_dst(&a1.into()).await.unwrap();
    assert_eq!(peers.get_dst_by_id(id).await, Some(a1.into()));
}
