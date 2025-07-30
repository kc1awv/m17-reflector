use serde::Serialize;
use std::collections::HashMap;
use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub callsign: String,
    pub module: char,
    pub connected_since: SystemTime,
    pub last_seen: SystemTime,
    pub packets_in: u64,
    pub bytes_in: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamInfo {
    pub source: String,
    pub destination: String,
    pub peer: String,
    pub module: char,
    pub stream_id: u16,
    pub frames: u32,
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleStats {
    pub module: char,
    pub clients: usize,
    pub active_streams: usize,
    pub total_packets: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsSnapshot {
    pub reflector_name: String,
    pub uptime_seconds: u64,
    pub total_clients: usize,
    pub total_streams: usize,
    pub total_packets: u64,
    pub total_bytes: u64,
    pub modules: Vec<ModuleStats>,
    pub clients: Vec<ClientInfo>,
    pub active_streams: Vec<StreamInfo>,
    pub recent_streams: Vec<StreamInfo>,
}

pub struct ReflectorState {
    pub start_time: Instant,
    pub name: String,
    pub clients: HashMap<String, ClientInfo>,
    pub active_streams: Vec<StreamInfo>,
    pub recent_streams: Vec<StreamInfo>,
}

impl ReflectorState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            name: String::new(),
            clients: HashMap::new(),
            active_streams: Vec::new(),
            recent_streams: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        let uptime_seconds = self.start_time.elapsed().as_secs();

        let mut module_map: HashMap<char, (usize, usize, u64, u64)> = HashMap::new();

        for client in self.clients.values() {
            let entry = module_map.entry(client.module).or_insert((0, 0, 0, 0));
            entry.0 += 1;
            entry.2 += client.packets_in;
            entry.3 += client.bytes_in;
        }

        for stream in &self.active_streams {
            if let Some(entry) = module_map.get_mut(&stream.module) {
                entry.1 += 1;
            } else {
                module_map.insert(stream.module, (0, 1, 0, 0));
            }
        }

        let module_stats: Vec<ModuleStats> = module_map
            .into_iter()
            .map(
                |(module, (clients, active_streams, total_packets, total_bytes))| ModuleStats {
                    module,
                    clients,
                    active_streams,
                    total_packets,
                    total_bytes,
                },
            )
            .collect();

        let (total_packets, total_bytes) = module_stats.iter().fold((0u64, 0u64), |acc, m| {
            (acc.0 + m.total_packets, acc.1 + m.total_bytes)
        });

        StatsSnapshot {
            reflector_name: self.name.clone(),
            uptime_seconds,
            total_clients: self.clients.len(),
            total_streams: self.active_streams.len(),
            total_packets,
            total_bytes,
            modules: module_stats,
            clients: self.clients.values().cloned().collect(),
            active_streams: self.active_streams.clone(),
            recent_streams: self.recent_streams.clone(),
        }
    }
}
