use crate::config::Config;
use crate::reflector::Reflector;
use crate::packet::{parse_packet, Packet};
use crate::control::{handle_control_packet, send_ping, send_disc, send_conn};
use crate::router::route_stream_packet;

use tokio::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use std::time::{Duration, Instant};
use log::{info, warn, error};

pub async fn run_with_state(
    config: &Config,
    reflector: Arc<Mutex<Reflector>>,
    tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(&config.bind_address).await?);
    info!("Reflector listening on {}", &config.bind_address);

    if config.strict_crc {
        info!("CRC enforcement mode: STRICT");
    } else {
        info!("CRC enforcement mode: PERMISSIVE");
    }

    for link in &config.interlinks {
        let addr: std::net::SocketAddr = link.address.parse()?;
        for module in &link.modules {
            {
                let mut r = reflector.lock().await;
                r.add_link_peer(*module, link.name.clone(), addr);
            }
            send_conn(&config.reflector_name, *module, addr, &socket).await?;
        }
    }

    tokio::spawn(run_keepalive_task(
        Arc::clone(&reflector),
        Arc::clone(&socket),
    ));
    tokio::spawn(run_stream_timeout_task(Arc::clone(&reflector), tx.clone()));

    let mut buf = [0u8; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let data = &buf[..len];

        match parse_packet(data) {
            Ok(Packet::Control(ctrl)) => {
                let mut r = reflector.lock().await;
                if let Err(e) = handle_control_packet(ctrl, addr, &mut r, &socket).await {
                    error!("Error handling control packet: {}", e);
                }
                let _ = tx.send("update".into());
            }
            Ok(Packet::Stream(stream)) => {
                route_stream_packet(
                    stream,
                    data,
                    addr,
                    &config.reflector_name,
                    &reflector,
                    &socket,
                    config.strict_crc,
                    tx.clone(),
                )
                .await;
            }
            Err(e) => {
                warn!("Invalid packet from {}: {:?}", addr, e);
            }
        }
    }
}

async fn run_keepalive_task(reflector: Arc<Mutex<Reflector>>, socket: Arc<UdpSocket>) {
    loop {
        {
            let mut r = reflector.lock().await;
            let now = Instant::now();

            let mut stale_peers = Vec::new();
            for module in r.modules.values() {
                for (addr, peer) in module.peers.iter() {
                    if now.duration_since(peer.last_seen) > Duration::from_secs(30) {
                        stale_peers.push(*addr);
                    }
                }
            }

            for module in r.modules.values() {
                for peer in module.peers.values() {
                    let _ = send_ping(peer, &socket).await;
                }
            }

            for addr in stale_peers {
                if let Some(peer) = r.find_peer_mut(&addr) {
                    let _ = send_disc(peer, &socket).await;
                    info!("Peer {} ({}) timed out, sent DISC", peer.callsign, addr);
                }
                r.remove_peer(&addr);
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn run_stream_timeout_task(reflector: Arc<Mutex<Reflector>>, tx: broadcast::Sender<String>) {
    loop {
        {
            let mut r = reflector.lock().await;
            let now = Instant::now();
            let mut ended = Vec::new();
            for (id, info) in r.active_streams.iter() {
                if now.duration_since(info.last_frame) > Duration::from_secs(1) {
                    ended.push(*id);
                }
            }

            for &id in &ended {
                r.end_stream(id);
                for module in r.modules.values_mut() {
                    for peer in module.peers.values_mut() {
                        if peer.receiving_unicast == Some(id) {
                            peer.receiving_unicast = None;
                        }
                    }
                }
            }
            if !ended.is_empty() {
                let _ = tx.send("update".into());
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}