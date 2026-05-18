//! Audio System — spatial audio, adaptive music, DSP, streaming, HRTF
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ AUDIO CLIP ═══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioClip {
    pub id:         String,
    pub path:       String,
    pub format:     AudioFormat,
    pub duration:   f32,
    pub channels:   u8,
    pub sample_rate:u32,
    pub bit_depth:  u16,
    pub loop_:      bool,
    pub loop_start: f32,
    pub loop_end:   f32,
    pub preload:    bool,
    pub streaming:  bool,
    pub size_bytes: u64,
    pub loaded:     bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AudioFormat { Wav, Ogg, Mp3, Flac, Opus, Aiff, Custom(String) }

// ═══ AUDIO SOURCE ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioSource {
    pub id:           String,
    pub entity_id:    String,
    pub clip_id:      Option<String>,
    pub state:        PlayState,
    pub volume:       f32,
    pub pitch:        f32,
    pub pan:          f32,        // -1=left, 0=center, 1=right (2D only)
    pub spatial:      bool,
    pub position:     [f32;3],
    pub velocity:     [f32;3],
    pub min_dist:     f32,
    pub max_dist:     f32,
    pub rolloff:      RolloffMode,
    pub doppler:      f32,
    pub reverb_zone:  Option<String>,
    pub priority:     u32,
    pub bus:          String,
    pub time:         f32,        // playback position in seconds
    pub play_on_awake:bool,
    pub ignore_pause: bool,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PlayState { Stopped, Playing, Paused, Fading{to:f32,secs:f32} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum RolloffMode { Linear, Logarithmic, Custom }

impl AudioSource {
    pub fn new(id:&str, entity_id:&str) -> Self {
        Self { id:id.to_string(), entity_id:entity_id.to_string(), clip_id:None,
               state:PlayState::Stopped, volume:1.0, pitch:1.0, pan:0.0,
               spatial:true, position:[0.0;3], velocity:[0.0;3],
               min_dist:1.0, max_dist:100.0, rolloff:RolloffMode::Logarithmic,
               doppler:1.0, reverb_zone:None, priority:128, bus:"SFX".to_string(),
               time:0.0, play_on_awake:false, ignore_pause:false }
    }
    pub fn play(&mut self) { self.state = PlayState::Playing; }
    pub fn pause(&mut self) { if self.state==PlayState::Playing { self.state=PlayState::Paused; } }
    pub fn stop(&mut self) { self.state=PlayState::Stopped; self.time=0.0; }
    pub fn fade_out(&mut self, secs:f32) { self.state=PlayState::Fading{to:0.0,secs}; }
    pub fn fade_in(&mut self, secs:f32) { self.state=PlayState::Fading{to:1.0,secs}; }
    pub fn is_playing(&self)->bool { self.state==PlayState::Playing }
    pub fn is_active(&self)->bool { !matches!(self.state,PlayState::Stopped) }
}

// ═══ AUDIO BUS ════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioBus {
    pub name:       String,
    pub volume:     f32,
    pub muted:      bool,
    pub solo:       bool,
    pub effects:    Vec<BusEffect>,
    pub send_to:    Option<String>,   // bus routing
    pub peak_db:    f32,
    pub rms_db:     f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BusEffect {
    pub name:    String,
    pub kind:    EffectKind,
    pub enabled: bool,
    pub params:  HashMap<String,f32>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EffectKind {
    Eq       { bands:u32 },
    Compressor{ threshold:f32, ratio:f32, attack:f32, release:f32 },
    Limiter  { ceiling:f32, release:f32 },
    Reverb   { room:f32, damp:f32, width:f32, wet:f32 },
    Delay    { time:f32, feedback:f32, wet:f32, sync:bool },
    Chorus   { rate:f32, depth:f32, wet:f32 },
    Flanger  { rate:f32, depth:f32, feedback:f32, wet:f32 },
    Distortion{drive:f32, tone:f32, wet:f32 },
    Bitcrusher{bits:u32, rate:f32 },
    Filter   { kind:FilterKind, freq:f32, resonance:f32 },
    Convolution{ ir_path:String, wet:f32 },  // Room impulse response
    Pitch    { semitones:f32 },
    Custom   { id:String },
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FilterKind { LowPass, HighPass, BandPass, Notch, AllPass }

impl AudioBus {
    pub fn new(name:&str) -> Self {
        Self { name:name.to_string(), volume:1.0, muted:false, solo:false,
               effects:Vec::new(), send_to:None, peak_db:-60.0, rms_db:-60.0 }
    }
    pub fn master() -> Self { Self::new("Master") }
    pub fn sfx()    -> Self { let mut b=Self::new("SFX"); b.send_to=Some("Master".to_string()); b }
    pub fn music()  -> Self { let mut b=Self::new("Music"); b.send_to=Some("Master".to_string()); b }
    pub fn voice()  -> Self { let mut b=Self::new("Voice"); b.send_to=Some("Master".to_string()); b }
    pub fn ambient()-> Self { let mut b=Self::new("Ambient"); b.send_to=Some("Master".to_string()); b }
}

// ═══ ADAPTIVE MUSIC ══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AdaptiveMusicSystem {
    pub enabled:       bool,
    pub current_state: String,
    pub target_state:  String,
    pub transition:    Option<MusicTransition>,
    pub states:        HashMap<String,MusicState>,
    pub stingers:      Vec<Stinger>,
    pub layers:        HashMap<String,MusicLayer>,
    pub bpm:           f32,
    pub bar_duration:  f32,      // seconds per bar
    pub current_bar:   u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MusicState {
    pub name:        String,
    pub stems:       Vec<String>,   // stem IDs that play in this state
    pub transitions: HashMap<String,TransitionRule>,
    pub intensity:   f32,
    pub loop_:       bool,
    pub intro:       Option<String>,
    pub outro:       Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TransitionRule {
    pub target:    String,
    pub mode:      TransitionMode,
    pub condition: Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TransitionMode { Immediate, NextBar, NextBeat, CrossFade{secs:f32}, AtLoop }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MusicTransition {
    pub from:     String,
    pub to:       String,
    pub mode:     TransitionMode,
    pub progress: f32,
    pub duration: f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MusicLayer {
    pub id:      String,
    pub clip_id: String,
    pub volume:  f32,
    pub muted:   bool,
    pub fade_in: f32,
    pub fade_out:f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Stinger {
    pub id:        String,
    pub clip_id:   String,
    pub trigger:   String,
    pub one_shot:  bool,
    pub played:    bool,
    pub delay:     f32,
    pub volume:    f32,
    pub bus:       String,
}

impl AdaptiveMusicSystem {
    pub fn new() -> Self {
        Self { enabled:true, current_state:"silence".to_string(), target_state:"silence".to_string(),
               transition:None, states:HashMap::new(), stingers:Vec::new(),
               layers:HashMap::new(), bpm:120.0, bar_duration:2.0, current_bar:0 }
    }
    pub fn transition_to(&mut self, state:&str, mode:TransitionMode) {
        if self.states.contains_key(state) {
            tracing::info!("Music transition: {} → {}", self.current_state, state);
            self.target_state = state.to_string();
            self.transition = Some(MusicTransition {
                from:self.current_state.clone(), to:state.to_string(),
                mode, progress:0.0, duration:2.0,
            });
        }
    }
    pub fn tick(&mut self, delta:f32) {
        if let Some(t) = &mut self.transition {
            t.progress += delta / t.duration;
            if t.progress >= 1.0 {
                self.current_state = t.to.clone();
                self.transition = None;
                tracing::debug!("Music state: {}", self.current_state);
            }
        }
    }
}

// ═══ LISTENER ═════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioListener {
    pub position:   [f32;3],
    pub forward:    [f32;3],
    pub up:         [f32;3],
    pub velocity:   [f32;3],
    pub hrtf:       bool,
    pub entity_id:  Option<String>,
}
impl Default for AudioListener {
    fn default() -> Self {
        Self { position:[0.0;3], forward:[0.0,0.0,-1.0], up:[0.0,1.0,0.0],
               velocity:[0.0;3], hrtf:true, entity_id:None }
    }
}

// ═══ REVERB ZONE ══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ReverbZone {
    pub id:          String,
    pub position:    [f32;3],
    pub radius:      f32,
    pub fade_dist:   f32,
    pub preset:      ReverbPreset,
    pub room_size:   f32,
    pub damping:     f32,
    pub wet:         f32,
    pub dry:         f32,
    pub width:       f32,
    pub ir_path:     Option<String>,   // custom impulse response
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ReverbPreset {
    None, SmallRoom, MediumRoom, LargeRoom, Hall, Cathedral,
    Cave, Forest, Outdoors, Plate, Spring, Bathroom,
    Dungeon, Arena, Custom,
}

impl ReverbZone {
    pub fn preset_params(preset:&ReverbPreset) -> (f32,f32,f32) {
        match preset {
            ReverbPreset::SmallRoom   => (0.3, 0.8, 0.3),
            ReverbPreset::MediumRoom  => (0.5, 0.6, 0.5),
            ReverbPreset::LargeRoom   => (0.7, 0.4, 0.6),
            ReverbPreset::Hall        => (0.85,0.3, 0.7),
            ReverbPreset::Cathedral   => (0.95,0.2, 0.8),
            ReverbPreset::Cave        => (0.9, 0.15,0.75),
            ReverbPreset::Forest      => (0.4, 0.9, 0.2),
            ReverbPreset::Outdoors    => (0.2, 0.95,0.1),
            ReverbPreset::Dungeon     => (0.8, 0.3, 0.6),
            ReverbPreset::Arena       => (0.9, 0.4, 0.7),
            _                         => (0.5, 0.5, 0.5),
        }
    }
}

// ═══ AUDIO MANAGER ════════════════════════════════════════════════
pub struct AudioManager {
    pub clips:     HashMap<String,AudioClip>,
    pub sources:   HashMap<String,AudioSource>,
    pub buses:     HashMap<String,AudioBus>,
    pub listener:  AudioListener,
    pub music:     AdaptiveMusicSystem,
    pub reverb_zones: Vec<ReverbZone>,
    pub master_volume: f32,
    pub paused:    bool,
    pub sample_rate:u32,
    pub buffer_size:u32,
    pub hrtf_enabled:bool,
    pub doppler_factor:f32,
    pub speed_of_sound:f32,
    pub total_played:  u64,
    pub active_source_limit: u32,
}

impl AudioManager {
    pub fn new() -> Self {
        let mut buses = HashMap::new();
        buses.insert("Master".to_string(), AudioBus::master());
        buses.insert("SFX".to_string(),    AudioBus::sfx());
        buses.insert("Music".to_string(),  AudioBus::music());
        buses.insert("Voice".to_string(),  AudioBus::voice());
        buses.insert("Ambient".to_string(),AudioBus::ambient());
        Self {
            clips:HashMap::new(), sources:HashMap::new(), buses,
            listener:AudioListener::default(), music:AdaptiveMusicSystem::new(),
            reverb_zones:Vec::new(), master_volume:1.0, paused:false,
            sample_rate:48000, buffer_size:512, hrtf_enabled:true,
            doppler_factor:1.0, speed_of_sound:343.0,
            total_played:0, active_source_limit:64,
        }
    }

    pub fn register_clip(&mut self, clip:AudioClip) {
        self.clips.insert(clip.id.clone(), clip);
    }

    pub fn play(&mut self, clip_id:&str, entity_id:&str, pos:[f32;3], vol:f32, bus:&str) -> String {
        let src_id = format!("src_{}_{}", clip_id, self.total_played);
        let mut src = AudioSource::new(&src_id, entity_id);
        src.clip_id  = Some(clip_id.to_string());
        src.position = pos;
        src.volume   = vol;
        src.bus      = bus.to_string();
        src.play();
        self.sources.insert(src_id.clone(), src);
        self.total_played += 1;
        src_id
    }

    pub fn play_2d(&mut self, clip_id:&str, vol:f32, bus:&str) -> String {
        let id = self.play(clip_id, "none", [0.0;3], vol, bus);
        if let Some(src) = self.sources.get_mut(&id) { src.spatial = false; }
        id
    }

    pub fn stop_all_on_entity(&mut self, entity_id:&str) {
        for src in self.sources.values_mut() {
            if src.entity_id == entity_id { src.stop(); }
        }
    }

    pub fn set_listener_position(&mut self, pos:[f32;3], fwd:[f32;3], up:[f32;3]) {
        self.listener.position = pos;
        self.listener.forward  = fwd;
        self.listener.up       = up;
    }

    pub fn tick(&mut self, delta:f32) {
        if self.paused { return; }
        self.music.tick(delta);
        // Tick active sources
        let listener_pos = self.listener.position;
        for src in self.sources.values_mut() {
            if !src.is_active() { continue; }
            // Advance playback time
            if src.state == PlayState::Playing { src.time += delta * src.pitch; }
            // Fade handling
            if let PlayState::Fading{to, secs} = src.state.clone() {
                src.volume += (to - src.volume) * (delta / secs.max(0.001)).min(1.0);
                if (src.volume - to).abs() < 0.01 {
                    if to <= 0.0 { src.stop(); }
                    else { src.state = PlayState::Playing; }
                }
            }
        }
        // Remove stopped one-shot sources
        self.sources.retain(|_,s| !matches!(s.state, PlayState::Stopped) || s.play_on_awake);
    }

    pub fn music_state(&self) -> &str { &self.music.current_state }
    pub fn transition_music(&mut self, state:&str) {
        self.music.transition_to(state, TransitionMode::NextBar);
    }
    pub fn active_sources(&self) -> usize { self.sources.values().filter(|s|s.is_active()).count() }
    pub fn clip_count(&self) -> usize { self.clips.len() }
    pub fn bus_count(&self) -> usize { self.buses.len() }
    pub fn set_bus_volume(&mut self, bus:&str, vol:f32) {
        if let Some(b) = self.buses.get_mut(bus) { b.volume = vol; }
    }
    pub fn mute_bus(&mut self, bus:&str, muted:bool) {
        if let Some(b) = self.buses.get_mut(bus) { b.muted = muted; }
    }
}

impl Default for AudioManager { fn default() -> Self { Self::new() } }
extern crate tracing;
