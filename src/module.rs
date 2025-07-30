use crate::peer::Peer;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct Module {
    pub name: char,
    pub peers: HashMap<SocketAddr, Peer>,
    pub stats: ModuleStats,
}

pub struct ModuleStats {
    pub total_streams: u64,
    pub total_frames: u64,
}

impl Module {
    pub fn new(name: char) -> Self {
        Self {
            name,
            peers: HashMap::new(),
            stats: ModuleStats {
                total_streams: 0,
                total_frames: 0,
            },
        }
    }
}
