use crate::callsign::base_callsign;
use crate::module::Module;
use crate::peer::Peer;
use crate::state::{ClientInfo, ReflectorState, StreamInfo as ApiStreamInfo};
use log::info;
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::time::{Instant, SystemTime};

pub struct Reflector {
    pub name: String,
    pub modules: HashMap<char, Module>,
    pub active_streams: HashMap<u16, StreamInfo>,
    pub recent_streams: VecDeque<StreamInfo>,
    pub start_time: Instant,
    pub user_map: HashMap<String, HashSet<SocketAddr>>,
}

pub struct StreamInfo {
    pub callsign: String,
    pub destination: String,
    pub module: char,
    pub start_time: Instant,
    pub peer_callsign: String,
    pub peer: SocketAddr,
    pub frames: u32,
    pub is_broadcast: bool,
    pub end_time: Option<Instant>,
}

impl Reflector {
    pub fn new(name: &str, mod_names: &[char]) -> Self {
        let mut modules = HashMap::new();
        for name in mod_names {
            modules.insert(*name, Module::new(*name));
        }
        Self {
            name: name.to_string(),
            modules,
            active_streams: HashMap::new(),
            recent_streams: VecDeque::with_capacity(50),
            start_time: Instant::now(),
            user_map: HashMap::new(),
        }
    }

    pub fn find_peer_mut(&mut self, addr: &SocketAddr) -> Option<&mut Peer> {
        for module in self.modules.values_mut() {
            if let Some(peer) = module.peers.get_mut(addr) {
                return Some(peer);
            }
        }
        None
    }

    pub fn find_peer(&self, addr: &SocketAddr) -> Option<&Peer> {
        for module in self.modules.values() {
            if let Some(peer) = module.peers.get(addr) {
                return Some(peer);
            }
        }
        None
    }

    pub fn add_link_peer(&mut self, module: char, name: String, addr: SocketAddr) {
        if let Some(m) = self.modules.get_mut(&module) {
            m.peers
                .entry(addr)
                .or_insert_with(|| Peer::new_link(name, addr));
        }
    }

    pub fn remove_peer(&mut self, addr: &SocketAddr) {
        for module in self.modules.values_mut() {
            module.peers.remove(addr);
        }
        for addrs in self.user_map.values_mut() {
            addrs.remove(addr);
        }
        self.user_map.retain(|_, s| !s.is_empty());
    }

    pub fn record_user(&mut self, callsign: &str, addr: SocketAddr) {
        let base = base_callsign(callsign);
        self.user_map
            .entry(base)
            .or_insert_with(HashSet::new)
            .insert(addr);
    }

    pub fn find_user_peers(&self, callsign: &str) -> Vec<SocketAddr> {
        let base = base_callsign(callsign);
        self.user_map
            .get(&base)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn record_stream_frame(
        &mut self,
        stream_id: u16,
        callsign: &str,
        module: char,
        destination: &str,
        peer: SocketAddr,
        is_broadcast: bool,
    ) -> bool {
        if let Some(mod_ref) = self.modules.get_mut(&module) {
            mod_ref.stats.total_frames += 1;
        }
        if !self.active_streams.contains_key(&stream_id) {
            if is_broadcast {
                if self
                    .active_streams
                    .values()
                    .any(|s| s.module == module && s.is_broadcast)
                {
                    return false;
                }
            }

            if let Some(mod_ref) = self.modules.get_mut(&module) {
                mod_ref.stats.total_streams += 1;
            }
            info!(
                "Stream start [{}]: {} -> {} on module {}",
                stream_id, callsign, destination, module
            );
            self.active_streams.insert(
                stream_id,
                StreamInfo {
                    callsign: callsign.to_string(),
                    destination: destination.to_string(),
                    module,
                    start_time: Instant::now(),
                    peer_callsign: self
                        .find_peer(&peer)
                        .map(|p| p.callsign.clone())
                        .unwrap_or_default(),
                    peer,
                    frames: 1,
                    is_broadcast,
                    end_time: None,
                },
            );
            true
        } else {
            if let Some(entry) = self.active_streams.get_mut(&stream_id) {
                entry.frames += 1;
            }
            true
        }
    }

    pub fn end_stream(&mut self, stream_id: u16) {
        if let Some(info) = self.active_streams.remove(&stream_id) {
            let duration = info.start_time.elapsed().as_secs_f32();
            info!(
                "Stream end [{}]: {} -> {} on module {} — {} frames, {:.2} sec",
                stream_id, info.callsign, info.destination, info.module, info.frames, duration
            );

            self.recent_streams.push_back(StreamInfo {
                callsign: info.callsign,
                destination: info.destination,
                module: info.module,
                peer_callsign: info.peer_callsign,
                peer: info.peer,
                start_time: info.start_time,
                frames: info.frames,
                is_broadcast: info.is_broadcast,
                end_time: Some(Instant::now()),
            });

            if self.recent_streams.len() > 50 {
                self.recent_streams.pop_front();
            }
        }
    }

    pub fn get_stats(&self) -> HashMap<char, (u64, u64)> {
        self.modules
            .iter()
            .map(|(&name, module)| {
                (
                    name,
                    (module.stats.total_streams, module.stats.total_frames),
                )
            })
            .collect()
    }

    pub fn export_state(&self) -> ReflectorState {
        let mut clients = HashMap::new();

        for (module_name, module) in &self.modules {
            for peer in module.peers.values() {
                let last_seen = SystemTime::now() - peer.last_seen.elapsed();

                clients.insert(
                    peer.address.to_string(),
                    ClientInfo {
                        callsign: peer.callsign.clone(),
                        module: *module_name,
                        connected_since: peer.connected_at,
                        last_seen,
                        packets_in: peer.packets_in,
                        bytes_in: peer.bytes_in,
                    },
                );
            }
        }

        let active_streams: Vec<ApiStreamInfo> = self
            .active_streams
            .iter()
            .map(|(id, s)| ApiStreamInfo {
                source: s.callsign.clone(),
                peer: if s.peer_callsign.is_empty() {
                    s.peer.to_string()
                } else {
                    s.peer_callsign.clone()
                },
                destination: s.destination.clone(),
                module: s.module,
                stream_id: *id,
                frames: s.frames,
                started_at: SystemTime::now() - s.start_time.elapsed(),
                ended_at: None,
            })
            .collect();

        let recent_streams: Vec<ApiStreamInfo> = self
            .recent_streams
            .iter()
            .map(|s| ApiStreamInfo {
                peer: if s.peer_callsign.is_empty() {
                    s.peer.to_string()
                } else {
                    s.peer_callsign.clone()
                },
                source: s.callsign.clone(),
                destination: s.destination.clone(),
                module: s.module,
                stream_id: 0,
                frames: s.frames,
                started_at: SystemTime::now() - s.start_time.elapsed(),
                ended_at: s.end_time.map(|t| SystemTime::now() - t.elapsed()),
            })
            .collect();

        ReflectorState {
            start_time: self.start_time,
            name: self.name.clone(),
            clients,
            active_streams,
            recent_streams,
        }
    }
}
