use crate::callsign::encode_callsign;
use crate::packet::ControlKind;
use crate::peer::Peer;
use crate::reflector::Reflector;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::net::UdpSocket;

pub async fn handle_control_packet(
    pkt: ControlKind,
    addr: SocketAddr,
    reflector: &mut Reflector,
    socket: &UdpSocket,
) -> std::io::Result<()> {
    match pkt {
        ControlKind::Conn { from, module } => {
            if let Some(m) = reflector.modules.get_mut(&module) {
                let peer = Peer::new(from.clone(), addr);
                m.peers.insert(addr, peer);

                let reply = b"ACKN";
                socket.send_to(reply, addr).await?;
                log::info!("{} connected to module {} from {}", from, module, addr);
            } else {
                let reply = b"NACK";
                socket.send_to(reply, addr).await?;
                log::warn!(
                    "{} tried to connect to invalid module {} from {}",
                    from,
                    module,
                    addr
                );
            }
        }

        ControlKind::Lstn { from, module } => {
            if let Some(m) = reflector.modules.get_mut(&module) {
                let peer = Peer::new_listen(from.clone(), addr);
                m.peers.insert(addr, peer);

                let reply = b"ACKN";
                socket.send_to(reply, addr).await?;
                log::info!("{} listening on module {} from {}", from, module, addr);
            } else {
                let reply = b"NACK";
                socket.send_to(reply, addr).await?;
                log::warn!(
                    "{} tried to listen on invalid module {} from {}",
                    from,
                    module,
                    addr
                );
            }
        }

        ControlKind::Ping { from } => {
            let mut reply = Vec::new();
            reply.extend_from_slice(b"PONG");
            reply.extend_from_slice(&encode_callsign(&from));
            socket.send_to(&reply, addr).await?;

            if let Some(peer) = reflector.find_peer_mut(&addr) {
                peer.last_seen = Instant::now();
            }

            log::debug!("PING from {} ({}) → PONG sent", from, addr);
        }

        ControlKind::Pong { from } => {
            if let Some(peer) = reflector.find_peer_mut(&addr) {
                peer.last_seen = Instant::now();
            }
            log::debug!("PONG received from {} ({})", from, addr);
        }

        ControlKind::Disc { from } => {
            let mut reply = Vec::new();
            reply.extend_from_slice(b"DISC");
            reply.extend_from_slice(&encode_callsign(&from));
            socket.send_to(&reply, addr).await?;

            reflector.remove_peer(&addr);
            log::info!("{} ({}) disconnected", from, addr);
        }

        ControlKind::Ackn => {
            log::debug!("ACKN received (unexpected as server)");
        }

        ControlKind::Nack => {
            log::debug!("NACK received (unexpected as server)");
        }
    }

    Ok(())
}

pub async fn send_ping(peer: &Peer, socket: &UdpSocket) -> std::io::Result<()> {
    let mut pkt = Vec::new();
    pkt.extend_from_slice(b"PING");
    pkt.extend_from_slice(&encode_callsign(&peer.callsign));
    socket.send_to(&pkt, peer.address).await?;
    Ok(())
}

pub async fn send_disc(peer: &Peer, socket: &UdpSocket) -> std::io::Result<()> {
    let mut pkt = Vec::new();
    pkt.extend_from_slice(b"DISC");
    pkt.extend_from_slice(&encode_callsign(&peer.callsign));
    socket.send_to(&pkt, peer.address).await?;
    Ok(())
}

pub async fn send_conn(
    callsign: &str,
    module: char,
    addr: SocketAddr,
    socket: &UdpSocket,
) -> std::io::Result<()> {
    let mut pkt = Vec::new();
    pkt.extend_from_slice(b"CONN");
    pkt.extend_from_slice(&encode_callsign(callsign));
    pkt.push(module as u8);
    socket.send_to(&pkt, addr).await?;
    Ok(())
}
