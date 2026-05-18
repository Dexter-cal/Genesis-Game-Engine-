//! Genesis Advanced Networking — rollback netcode, lobby, matchmaking, voice
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,BTreeMap,VecDeque};
use chrono::{DateTime,Utc};

// ── Rollback Netcode ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PlayerInput {
    pub player_id:u32, pub frame:u64, pub flags:u32,
    pub aim_x:f32, pub aim_y:f32, pub move_x:f32, pub move_y:f32,
    pub predicted:bool, pub ts:DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct StateSnapshot { pub frame:u64, pub data:Vec<u8>, pub checksum:u64, pub ts:DateTime<Utc> }

#[derive(Debug,Clone)]
pub struct RollbackState {
    pub frame:u64, pub confirmed_frame:u64,
    pub rollback_frame:Option<u64>,
    pub input_buffer:BTreeMap<u64,HashMap<u32,PlayerInput>>,
    pub snapshots:VecDeque<StateSnapshot>,
    pub rollback_count:u64, pub desync_detected:bool,
}

impl RollbackState {
    pub fn new() -> Self {
        Self { frame:0, confirmed_frame:0, rollback_frame:None,
               input_buffer:BTreeMap::new(), snapshots:VecDeque::new(),
               rollback_count:0, desync_detected:false }
    }
    pub fn record_input(&mut self, frame:u64, player_id:u32, input:PlayerInput) {
        self.input_buffer.entry(frame).or_default().insert(player_id, input);
        let cutoff = frame.saturating_sub(20);
        self.input_buffer.retain(|f,_| *f >= cutoff);
    }
    pub fn save_snapshot(&mut self, data:Vec<u8>) {
        let checksum:u64 = data.iter().enumerate().map(|(i,&b)| b as u64*(i as u64+1)).sum();
        self.snapshots.push_back(StateSnapshot{frame:self.frame,data,checksum,ts:Utc::now()});
        while self.snapshots.len() > 16 { self.snapshots.pop_front(); }
    }
    pub fn advance(&mut self) { self.frame += 1; }
    pub fn confirm(&mut self, f:u64) { self.confirmed_frame = f; }
    pub fn needs_rollback(&self) -> bool { self.rollback_frame.is_some() }
    pub fn rollback_to(&mut self) -> Option<u64> { self.rollback_frame.take() }
    pub fn get_snapshot(&self, frame:u64) -> Option<&StateSnapshot> {
        self.snapshots.iter().find(|s| s.frame == frame)
    }
}

// ── Lobby ────────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Lobby {
    pub id:String, pub name:String, pub host_id:String,
    pub game_mode:String, pub map:String,
    pub state:LobbyState, pub visibility:LobbyVisibility,
    pub invite_code:Option<String>, pub password:Option<String>,
    pub max_players:u32, pub members:Vec<LobbyMember>,
    pub config:HashMap<String,serde_json::Value>,
    pub chat:Vec<LobbyMsg>, pub created_at:DateTime<Utc>,
    pub region:String, pub ranked:bool,
}

impl Lobby {
    pub fn new(host_id:&str, host_name:&str, mode:&str, max:u32, vis:LobbyVisibility) -> Self {
        let invite = if matches!(vis, LobbyVisibility::InviteOnly) {
            Some(format!("{:06X}", Utc::now().timestamp_millis() as u32))
        } else { None };
        Self {
            id: format!("lobby_{}", Utc::now().timestamp_millis()),
            name: format!("{}'s Lobby", host_name),
            host_id:host_id.to_string(), game_mode:mode.to_string(),
            map:"default".to_string(), state:LobbyState::Open,
            visibility:vis, invite_code:invite, password:None,
            max_players:max, members:Vec::new(),
            config:HashMap::new(), chat:Vec::new(),
            created_at:Utc::now(), region:"auto".to_string(), ranked:false,
        }
    }
    pub fn is_full(&self)->bool{ self.members.len() as u32 >= self.max_players }
    pub fn is_ready(&self)->bool{ !self.members.is_empty() && self.members.iter().all(|m|m.ready) }
    pub fn has_member(&self,id:&str)->bool{ self.members.iter().any(|m|m.player_id==id) }
    pub fn add_member(&mut self, m:LobbyMember) {
        let name = m.player_name.clone();
        self.members.push(m);
        self.chat.push(LobbyMsg{sender:"system".to_string(),content:format!("{} joined",name),ts:Utc::now()});
    }
    pub fn remove_member(&mut self, id:&str) {
        self.members.retain(|m| m.player_id != id);
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum LobbyState { Open, Starting{countdown:f32}, InGame, PostGame, Closed }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum LobbyVisibility { Public, FriendsOnly, InviteOnly, Private }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LobbyMember {
    pub player_id:String, pub player_name:String,
    pub team:Option<u32>, pub ready:bool, pub ping_ms:u32,
    pub role:MemberRole, pub joined_at:DateTime<Utc>,
    pub mmr:f32, pub is_ai:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MemberRole { Host, Mod, Member, Spectator }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LobbyMsg { pub sender:String, pub content:String, pub ts:DateTime<Utc> }

// ── Matchmaking ──────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MatchmakingTicket {
    pub id:String, pub player_ids:Vec<String>, pub queue:String,
    pub mmr:f32, pub region:Option<String>, pub created:DateTime<Utc>,
    pub status:TicketStatus, pub wait_secs:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TicketStatus { Searching, Found{match_id:String}, TimedOut, Cancelled }

// ── Connected Player ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ConnectedPlayer {
    pub player_id:String, pub session_id:String, pub display_name:String,
    pub ip:String, pub region:String, pub rtt_ms:f32,
    pub packet_loss:f32, pub bytes_sent:u64, pub bytes_recv:u64,
    pub connected_at:DateTime<Utc>, pub lobby_id:Option<String>,
    pub state:ConnState,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ConnState { Connecting, Connected, InLobby, InGame, Disconnected{reason:String} }

// ── Voice Chat ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VoiceConfig {
    pub enabled:bool, pub mode:VoiceMode, pub codec:String,
    pub bitrate_kbps:u32, pub ptt:bool, pub vad:bool,
    pub noise_suppress:bool, pub proximity:bool,
    pub proximity_range_m:f32, pub team_voice:bool,
}
impl Default for VoiceConfig {
    fn default() -> Self {
        Self { enabled:true, mode:VoiceMode::PushToTalk, codec:"opus".to_string(),
               bitrate_kbps:32, ptt:true, vad:true, noise_suppress:true,
               proximity:true, proximity_range_m:30.0, team_voice:true }
    }
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum VoiceMode { Always, PushToTalk, VoiceActivity }

// ── Network Manager ──────────────────────────────────────────────
pub struct NetworkManager {
    pub rollback:RollbackState, pub max_rollback_frames:u32, pub input_delay:u32,
    pub lobbies:HashMap<String,Lobby>, pub tickets:Vec<MatchmakingTicket>,
    pub players:HashMap<String,ConnectedPlayer>, pub voice:VoiceConfig,
    pub is_server:bool, pub local_player_id:Option<String>,
    pub current_lobby:Option<String>, pub ping_ms:f32, pub packet_loss:f32,
    pub upload_bytes:u64, pub download_bytes:u64,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            rollback:RollbackState::new(), max_rollback_frames:8, input_delay:2,
            lobbies:HashMap::new(), tickets:Vec::new(), players:HashMap::new(),
            voice:VoiceConfig::default(), is_server:false, local_player_id:None,
            current_lobby:None, ping_ms:0.0, packet_loss:0.0,
            upload_bytes:0, download_bytes:0,
        }
    }
    pub fn create_lobby(&mut self, host_id:&str, host_name:&str, mode:&str, max:u32, vis:LobbyVisibility) -> String {
        let lobby = Lobby::new(host_id, host_name, mode, max, vis);
        let id = lobby.id.clone();
        self.lobbies.insert(id.clone(), lobby);
        self.current_lobby = Some(id.clone());
        tracing::info!("Created lobby: {}", id);
        id
    }
    pub fn join_lobby(&mut self, lobby_id:&str, member:LobbyMember) -> Result<(),String> {
        let lobby = self.lobbies.get_mut(lobby_id).ok_or("Lobby not found")?;
        if lobby.is_full() { return Err("Lobby full".to_string()); }
        if matches!(lobby.state, LobbyState::InGame) { return Err("Game in progress".to_string()); }
        lobby.add_member(member);
        Ok(())
    }
    pub fn leave_lobby(&mut self, lobby_id:&str, player_id:&str) {
        if let Some(lobby) = self.lobbies.get_mut(lobby_id) {
            lobby.remove_member(player_id);
        }
    }
    pub fn record_input(&mut self, frame:u64, player_id:u32, input:PlayerInput) {
        self.rollback.record_input(frame, player_id, input);
    }
    pub fn advance_frame(&mut self) { self.rollback.advance(); }
    pub fn confirm_frame(&mut self, f:u64) { self.rollback.confirm(f); }
    pub fn lobby_count(&self) -> usize { self.lobbies.len() }
    pub fn player_count(&self) -> usize { self.players.len() }
}
extern crate tracing;
