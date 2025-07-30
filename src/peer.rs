use std::net::SocketAddr;

pub struct Peer {
    pub callsign: String,
    pub address: SocketAddr,
    pub connected_at: std::time::SystemTime,
    pub last_seen: std::time::Instant,
    pub packets_in: u64,
    pub packets_out: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub is_link: bool,
    pub listen_only: bool,
    pub receiving_unicast: Option<u16>,
}

impl Peer {
    pub fn new(callsign: String, address: SocketAddr) -> Self {
        Self {
            callsign,
            address,
            connected_at: std::time::SystemTime::now(),
            last_seen: std::time::Instant::now(),
            packets_in: 0,
            packets_out: 0,
            bytes_in: 0,
            bytes_out: 0,
            is_link: false,
            listen_only: false,
            receiving_unicast: None,
        }
    }

    pub fn new_link(callsign: String, address: SocketAddr) -> Self {
        let mut peer = Self::new(callsign, address);
        peer.is_link = true;
        peer
    }

    pub fn new_listen(callsign: String, address: SocketAddr) -> Self {
        let mut peer = Self::new(callsign, address);
        peer.listen_only = true;
        peer
    }

    pub fn increment_rx(&mut self, bytes: usize) {
        self.packets_in += 1;
        self.bytes_in += bytes as u64;
        self.last_seen = std::time::Instant::now();
    }

    pub fn increment_tx(&mut self, bytes: usize) {
        self.packets_out += 1;
        self.bytes_out += bytes as u64;
    }
}
