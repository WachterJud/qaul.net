//! Automatically forward a port to qaul-hubd to allow easy reverse
//! connections form a public router.

use igd::{search_gateway, Gateway, PortMappingProtocol as Protocol};
use ipnetwork::IpNetwork;
use pnet::datalink;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use tracing::trace;

//fn ip_is_local(ip: IpV4Addr)

/// Check if an IP is in one of the local IP ranges
///
/// We perform this check to see if an IP address _could_ be one that
/// a gateway can reach.  This auto-detection can go wrong though if
/// there are multiple address spaces availabe (for example via a VPN)
fn check_local(ip: &Ipv4Addr) -> bool {
    let [a, b, c, d] = ip.octets();
    match (a, b, c, d) {
        (10, _, _, _) => true,
        (192, 168, _, _) => true,
        (172, n, _, _) if n > 16 && n < 31 => true,
        (_, _, _, _) => false,
    }
}

fn find_local_ip() -> Option<Ipv4Addr> {
    datalink::interfaces()
        .into_iter()
        .map(|_if| _if.ips)
        .fold(None, |res, ips| {
            let mut new = res;
            for ip in ips {
                use IpNetwork::*;
                match (res, ip) {
                    (None, V4(n)) if check_local(&n.ip()) => {
                        new = Some(n.ip());
                        break;
                    }
                    (_, _) => {}
                }
            }

            new
        })
}

pub(crate) fn open_port(port: u16) -> Option<()> {
    let gw = search_gateway(Default::default()).ok()?;

    let ip = gw.get_external_ip().ok()?;
    trace!("Publicly accessible via: {}", ip);

    let local_ip =
        find_local_ip().unwrap_or_else(|| crate::elog("Couldn't find IP to bind to", 128));
    trace!("Local ip: {}", local_ip);

    let local_addr = SocketAddrV4::new(local_ip, 8080);

    gw.add_port(Protocol::TCP, port, local_addr, 0, "qaul-hubd tcp driver")
        .unwrap_or_else(|e| crate::elog(format!("{:?}", e), 128));
    trace!("UPNP port {} opened with infinite lease!", port);
    Some(())
}
