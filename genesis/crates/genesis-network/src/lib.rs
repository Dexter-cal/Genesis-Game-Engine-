//! Network — transport layer, connection management, packet serialization, relay
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,VecDeque};
use chrono::{DateTime,Utc};

// ═══ TRANSPORT ════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Transport { Udp, Tcp, WebSocket{tls:bool}, WebRtc{stun:Vec<String>}, Quic, Custom(String) }

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum ConnState { Disconnected, Connecting, Connected, Authenticated, Disconnecting, Failed(String) }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Connection {
    pub id:          String,
    pub peer_id:     String,
    pub transport:   Transport,
    pub state:       ConnState,
    pub ip:          String,
    pub port:        u16,
    pub rtt_ms:      f32,
    pub packet_loss: f32,
    pub jitter_ms:   f32,
    pub bytes_sent:  u64,
    pub bytes_recv:  u64,
    pub pkts_sent:   u64,
    pub pkts_recv:   u64,
    pub pkts_dropped:u64,
    pub connected_at:Option<DateTime<Utc>>,
    pub last_recv:   Option<DateTime<Utc>>,
    pub timeout_secs:f32,
    pub ping_interval_secs:f32,
    pub relay_id:    Option<String>,
    pub encrypted:   bool,
    pub region:      String,
}

impl Connection {
    pub fn new(id:&str, peer:&str, ip:&str, port:u16, transport:Transport) -> Self {
        Self { id:id.to_string(), peer_id:peer.to_string(), transport, state:ConnState::Disconnected,
               ip:ip.to_string(), port, rtt_ms:0.0, packet_loss:0.0, jitter_ms:0.0,
               bytes_sent:0, bytes_recv:0, pkts_sent:0, pkts_recv:0, pkts_dropped:0,
               connected_at:None, last_recv:None, timeout_secs:30.0,
               ping_interval_secs:1.0, relay_id:None, encrypted:true, region:"auto".to_string() }
    }
    pub fn is_connected(&self) -> bool { matches!(self.state, ConnState::Connected|ConnState::Authenticated) }
    pub fn uptime_secs(&self) -> f64 { self.connected_at.map(|t|(Utc::now()-t).num_seconds() as f64).unwrap_or(0.0) }
    pub fn quality(&self) -> ConnectionQuality {
        if self.rtt_ms < 50.0 && self.packet_loss < 0.01 { ConnectionQuality::Excellent }
        else if self.rtt_ms < 120.0 && self.packet_loss < 0.05 { ConnectionQuality::Good }
        else if self.rtt_ms < 250.0 && self.packet_loss < 0.10 { ConnectionQuality::Fair }
        else { ConnectionQuality::Poor }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ConnectionQuality { Excellent, Good, Fair, Poor, Disconnected }

// ═══ PACKET ═══════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Packet {
    pub id:        u32,
    pub sequence:  u32,
    pub ack:       u32,
    pub ack_bits:  u32,
    pub channel:   Channel,
    pub kind:      PacketKind,
    pub payload:   Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub size:      u32,
    pub compressed:bool,
    pub encrypted: bool,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum Channel { Unreliable, Reliable, ReliableOrdered, UnreliableSequenced }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PacketKind {
    Ping, Pong, Connect, Accept, Reject(String), Disconnect(String),
    Input { frame:u64, data:Vec<u8> },
    State { frame:u64, delta:bool, data:Vec<u8> },
    Event { kind:String, data:Vec<u8> },
    RPC   { method:String, args:Vec<u8>, id:u32 },
    RPCResponse { id:u32, result:Vec<u8>, error:Option<String> },
    Chat  { message:String, sender:String, channel:String },
    Relay { to:String, data:Vec<u8> },
    Custom{ type_id:u16, data:Vec<u8> },
}

// ═══ ACK SYSTEM ══════════════════════════════════════════════════
#[derive(Debug,Clone)]
pub struct AckBuffer {
    pub local_seq:   u32,
    pub remote_seq:  u32,
    pub ack_bits:    u32,
    pub sent:        VecDeque<(u32, DateTime<Utc>)>,
    pub received:    VecDeque<u32>,
    pub rtt_samples: VecDeque<f32>,
    pub capacity:    usize,
}
impl AckBuffer {
    pub fn new() -> Self {
        Self { local_seq:0, remote_seq:0, ack_bits:0, sent:VecDeque::new(),
               received:VecDeque::new(), rtt_samples:VecDeque::new(), capacity:128 }
    }
    pub fn next_seq(&mut self) -> u32 { self.local_seq = self.local_seq.wrapping_add(1); self.local_seq }
    pub fn record_sent(&mut self, seq:u32) {
        self.sent.push_back((seq, Utc::now()));
        while self.sent.len() > self.capacity { self.sent.pop_front(); }
    }
    pub fn record_recv(&mut self, seq:u32) {
        self.received.push_back(seq);
        while self.received.len() > self.capacity { self.received.pop_front(); }
    }
    pub fn acknowledge(&mut self, seq:u32) {
        if let Some(pos) = self.sent.iter().position(|(s,_)|*s==seq) {
            let (_, sent_at) = self.sent.remove(pos).unwrap();
            let rtt = (Utc::now()-sent_at).num_milliseconds() as f32;
            self.rtt_samples.push_back(rtt);
            while self.rtt_samples.len() > 32 { self.rtt_samples.pop_front(); }
        }
    }
    pub fn avg_rtt(&self) -> f32 {
        if self.rtt_samples.is_empty() { 0.0 }
        else { self.rtt_samples.iter().sum::<f32>() / self.rtt_samples.len() as f32 }
    }
}

// ═══ RELAY SERVER ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct RelayServer { pub id:String, pub url:String, pub region:String, pub ping_ms:u32, pub active:bool }

// ═══ NAT TRAVERSAL ═══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum NatType { Open, Moderate, Strict, Symmetric }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NatInfo { pub kind:NatType, pub external_ip:String, pub external_port:u16 }

// ═══ BANDWIDTH THROTTLE ══════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BandwidthConfig {
    pub max_send_kbps:    u32,
    pub max_recv_kbps:    u32,
    pub priority_queues:  u32,
    pub compression:      bool,
    pub compression_level:u32,
    pub encryption:       bool,
}
impl Default for BandwidthConfig {
    fn default() -> Self { Self { max_send_kbps:1024, max_recv_kbps:4096, priority_queues:4, compression:true, compression_level:3, encryption:true } }
}

// ═══ NETWORK MANAGER ═════════════════════════════════════════════
pub struct NetworkManager {
    pub connections:   HashMap<String,Connection>,
    pub relay_servers: Vec<RelayServer>,
    pub nat:           Option<NatInfo>,
    pub bandwidth:     BandwidthConfig,
    pub pending_packets: VecDeque<(String,Packet)>,
    pub recv_queue:    VecDeque<(String,Packet)>,
    pub ack_buffers:   HashMap<String,AckBuffer>,
    pub total_sent:    u64,
    pub total_recv:    u64,
    pub bytes_sent:    u64,
    pub bytes_recv:    u64,
    pub is_server:     bool,
    pub listen_port:   u16,
    pub max_clients:   u32,
    pub tick_rate:     u32,
    pub region:        String,
}

impl NetworkManager {
    pub fn new(is_server:bool, port:u16, max_clients:u32) -> Self {
        Self {
            connections:HashMap::new(), relay_servers:Self::default_relays(),
            nat:None, bandwidth:BandwidthConfig::default(),
            pending_packets:VecDeque::new(), recv_queue:VecDeque::new(),
            ack_buffers:HashMap::new(), total_sent:0, total_recv:0,
            bytes_sent:0, bytes_recv:0, is_server, listen_port:port,
            max_clients, tick_rate:20, region:"auto".to_string(),
        }
    }

    fn default_relays() -> Vec<RelayServer> {
        vec![
            RelayServer { id:"relay-us-east".to_string(), url:"relay-us-east.genesis-engine.io:7777".to_string(), region:"us-east".to_string(), ping_ms:0, active:true },
            RelayServer { id:"relay-eu-west".to_string(), url:"relay-eu-west.genesis-engine.io:7777".to_string(), region:"eu-west".to_string(), ping_ms:0, active:true },
            RelayServer { id:"relay-ap-south".to_string(),url:"relay-ap-south.genesis-engine.io:7777".to_string(),region:"ap-south".to_string(),ping_ms:0, active:true },
        ]
    }

    pub fn connect(&mut self, peer_id:&str, ip:&str, port:u16, transport:Transport) -> String {
        let id = format!("conn_{}",self.connections.len());
        let mut conn = Connection::new(&id, peer_id, ip, port, transport);
        conn.state = ConnState::Connecting;
        tracing::info!("Connecting to {}:{} ({})", ip, port, peer_id);
        self.ack_buffers.insert(id.clone(), AckBuffer::new());
        self.connections.insert(id.clone(), conn);
        id
    }

    pub fn disconnect(&mut self, conn_id:&str, reason:&str) {
        if let Some(c) = self.connections.get_mut(conn_id) {
            c.state = ConnState::Disconnecting;
            tracing::info!("Disconnecting {} — {}", conn_id, reason);
        }
        self.connections.remove(conn_id);
        self.ack_buffers.remove(conn_id);
    }

    pub fn send(&mut self, conn_id:&str, packet:Packet) {
        if let Some(ack) = self.ack_buffers.get_mut(conn_id) { ack.record_sent(packet.sequence); }
        self.bytes_sent += packet.size as u64;
        self.total_sent += 1;
        self.pending_packets.push_back((conn_id.to_string(), packet));
    }

    pub fn broadcast(&mut self, packet:Packet) {
        let ids:Vec<_>=self.connections.keys().cloned().collect();
        for id in ids { self.send(&id, packet.clone()); }
    }

    pub fn poll(&mut self) -> Option<(String,Packet)> { self.recv_queue.pop_front() }

    pub fn tick(&mut self, _delta:f32) {
        // Update RTT from ack buffers
        for (conn_id, ack) in &self.ack_buffers {
            if let Some(conn) = self.connections.get_mut(conn_id) {
                conn.rtt_ms = ack.avg_rtt();
            }
        }
        // Check timeouts
        let now = Utc::now();
        let timed_out:Vec<_> = self.connections.iter()
            .filter(|(_,c)| c.is_connected())
            .filter(|(_, c)| c.last_recv.map(|t|(now-t).num_seconds() as f32 > c.timeout_secs).unwrap_or(false))
            .map(|(id,_)| id.clone())
            .collect();
        for id in timed_out { tracing::warn!("Connection timed out: {}", id); self.disconnect(&id, "timeout"); }
    }

    pub fn connected_count(&self) -> usize { self.connections.values().filter(|c|c.is_connected()).count() }
    pub fn connection_count(&self) -> usize { self.connections.len() }
    pub fn is_full(&self) -> bool { self.connected_count() >= self.max_clients as usize }
    pub fn avg_rtt(&self) -> f32 {
        let conns:Vec<_>=self.connections.values().filter(|c|c.is_connected()).collect();
        if conns.is_empty() { return 0.0; }
        conns.iter().map(|c|c.rtt_ms).sum::<f32>() / conns.len() as f32
    }
}
extern crate tracing;
