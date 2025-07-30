use crate::packet::StreamPacket;
use crate::reflector::Reflector;
use log::{debug, error, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, broadcast};

pub async fn route_stream_packet(
    stream: StreamPacket,
    data: &[u8],
    addr: SocketAddr,
    reflector_name: &str,
    reflector: &Arc<Mutex<Reflector>>,
    socket: &Arc<UdpSocket>,
    strict_crc: bool,
    tx: broadcast::Sender<String>,
) {
    if !stream.crc_ok {
        debug!(
            "Forwarding packet with bad CRC: {} -> {} (stream {})",
            stream.src, stream.dst, stream.stream_id
        );
    }

    if strict_crc && !stream.crc_ok {
        error!(
            "Packet with bad CRC dropped: {} -> {} (stream {})",
            stream.src, stream.dst, stream.stream_id
        );
        return;
    }

    if stream.src == "INVALID" || stream.dst == "INVALID" {
        warn!("Dropped invalid address stream packet");
        return;
    }
    if stream.dst.starts_with("RESERVED-") || stream.src.starts_with("RESERVED-") {
        warn!("Dropped reserved address stream packet");
        return;
    }

    let mut r = reflector.lock().await;

    let sender_module = r
        .modules
        .values()
        .find(|m| m.peers.contains_key(&addr))
        .map(|m| m.name);

    if sender_module.is_none() {
        warn!(
            "Stream from {} ({}) dropped: sender not registered",
            stream.src, addr
        );
        return;
    }
    let sender_module = sender_module.unwrap();

    if let Some(peer) = r.find_peer(&addr) {
        if peer.listen_only {
            warn!(
                "Dropped stream from listen-only peer {} ({})",
                peer.callsign, addr
            );
            return;
        }
    }

    let reflector_call = format!("{} {}", reflector_name, sender_module);
    let is_broadcast = 
        stream.dst == "BROADCAST" || stream.dst == "ALL" || stream.dst == reflector_call;

    if let Some(peer) = r.find_peer_mut(&addr) {
        peer.increment_rx(data.len());
    }

    r.record_user(&stream.src, addr);

    let is_new = !r.active_streams.contains_key(&stream.stream_id);
    let allowed = r.record_stream_frame(
        stream.stream_id,
        &stream.src,
        sender_module,
        &stream.dst,
        addr,
        is_broadcast,
    );
    if !allowed {
        warn!(
            "Stream from {} ignored: module {} already has an active stream",
            stream.src, sender_module
        );
        return;
    }
    let _ = tx.send("update".into());

    let peer_addresses: Vec<SocketAddr> = if is_broadcast {
        if let Some(module) = r.modules.get(&sender_module) {
            module.peers.keys().cloned().collect()
        } else {
            Vec::new()
        }
    } else {
        let addrs = r.find_user_peers(&stream.dst);
        if addrs.is_empty() {
            warn!(
                "Stream from {} to {} dropped: destination unknown",
                stream.src, stream.dst
            );
        }
        addrs
    };

    let sender_is_link = r.find_peer(&addr).map(|p| p.is_link).unwrap_or(false);

    for peer_addr in peer_addresses {
        if peer_addr != addr {
            if sender_is_link {
                if let Some(p) = r.find_peer(&peer_addr) {
                    if p.is_link {
                        continue;
                    }
                }
            }

            if is_broadcast {
                if let Some(p) = r.find_peer(&peer_addr) {
                    if p.receiving_unicast.is_some() {
                        continue;
                    }
                }
            }

            let _ = socket.send_to(data, peer_addr).await;

            if let Some(p) = r.find_peer_mut(&peer_addr) {
                p.increment_tx(data.len());
                if !is_broadcast && is_new {
                    p.receiving_unicast = Some(stream.stream_id);
                }
                if !is_broadcast && stream.last_frame {
                    if p.receiving_unicast == Some(stream.stream_id) {
                        p.receiving_unicast = None;
                    }
                }
            }
        }
    }

    if stream.last_frame {
        r.end_stream(stream.stream_id);
        let _ = tx.send("update".into());
    }
}
