//! Genesis Cinematics — cutscenes, AI director, poster gen, credits, trailer editor
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Cutscene ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Cutscene {
    pub id:String, pub name:String, pub duration_secs:f32,
    pub skippable:bool, pub skip_after_secs:f32,
    pub shots:Vec<Shot>, pub actors:Vec<ActorDir>,
    pub audio_cues:Vec<AudioCue>, pub subtitle_tracks:Vec<SubtitleTrack>,
    pub effects:Vec<CinematicEffect>, pub state_changes:Vec<StateChange>,
    pub tags:Vec<String>, pub created_by:String, pub version:u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Shot {
    pub id:String, pub start:f32, pub end:f32,
    pub camera:CineCam, pub kind:ShotKind,
    pub transition_in:Transition, pub transition_out:Transition,
    pub ai_directed:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CineCam {
    pub pos_keys:Vec<CamKey>, pub target_keys:Vec<CamKey>,
    pub fov_keys:Vec<FloatKey>, pub aperture_keys:Vec<FloatKey>,
    pub focus_keys:Vec<FloatKey>,
    pub follow:Option<String>, pub look_at:Option<String>,
    pub shake:Option<ShakeProfile>, pub lens:LensProfile,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CamKey { pub t:f32, pub pos:[f32;3], pub ease:EaseKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct FloatKey { pub t:f32, pub v:f32, pub ease:EaseKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EaseKind { Linear, EaseIn, EaseOut, EaseInOut, Spring, Bounce }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ShakeProfile { pub intensity:f32, pub freq:f32, pub duration:f32, pub decay:f32 }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LensProfile {
    pub focal_mm:f32, pub aperture_fstop:f32, pub sensor_mm:f32,
    pub distortion:f32, pub chromatic:f32, pub vignette:f32,
    pub flare:bool, pub anamorphic:bool,
}
impl LensProfile {
    pub fn cinema_50mm()->Self{Self{focal_mm:50.0,aperture_fstop:2.0,sensor_mm:36.0,distortion:-0.02,chromatic:0.01,vignette:0.15,flare:true,anamorphic:false}}
    pub fn wide_24mm()->Self{Self{focal_mm:24.0,aperture_fstop:2.8,sensor_mm:36.0,distortion:-0.06,chromatic:0.02,vignette:0.25,flare:true,anamorphic:false}}
    pub fn tele_200mm()->Self{Self{focal_mm:200.0,aperture_fstop:4.0,sensor_mm:36.0,distortion:0.01,chromatic:0.005,vignette:0.08,flare:false,anamorphic:false}}
    pub fn anamorphic_40mm()->Self{Self{focal_mm:40.0,aperture_fstop:1.8,sensor_mm:36.0,distortion:0.04,chromatic:0.03,vignette:0.3,flare:true,anamorphic:true}}
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ShotKind {
    ExtremeWide,Wide,MediumWide,Medium,MediumClose,CloseUp,ExtremeCloseUp,
    Dolly{dir:[f32;3]},Crane{arc_deg:f32},Pan{dir:f32},Tilt{dir:f32},
    Tracking{entity:String},Handheld{shake:f32},Drone{alt_m:f32},
    Pov{character:String},OverShoulder{from:String,to:String},
    Dutch{angle:f32},BulletTime{scale:f32},Zolly{dolly_in:bool},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Transition { pub kind:TransKind, pub duration:f32 }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TransKind { Cut,Dissolve,Fade{color:[f32;4]},Wipe{dir:String},Glitch,FilmBurn }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ActorDir {
    pub entity_id:String, pub marks:Vec<ActorMark>,
    pub dialogue:Vec<DialogueLine>, pub animations:Vec<AnimCue>,
    pub expressions:Vec<ExpressionCue>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ActorMark { pub t:f32, pub pos:[f32;3], pub rot:[f32;3], pub speed:f32, pub style:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueLine {
    pub start:f32, pub text:String, pub voice_file:Option<String>,
    pub emotion:String, pub intensity:f32,
    pub translations:HashMap<String,String>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AnimCue { pub t:f32, pub anim_id:String, pub blend:f32, pub speed:f32, pub loop_:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExpressionCue { pub t:f32, pub shapes:HashMap<String,f32>, pub blend_secs:f32 }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioCue { pub t:f32, pub kind:AudioCueKind, pub volume:f32, pub fade_in:f32, pub fade_out:f32 }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AudioCueKind { Music{track:String,loop_:bool}, Sfx{clip:String}, Silence{dur:f32}, Ambience{scene:String} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SubtitleTrack { pub lang:String, pub rtl:bool, pub entries:Vec<SubEntry> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SubEntry { pub start:f32, pub end:f32, pub text:String, pub speaker:Option<String> }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CinematicEffect { pub start:f32, pub end:f32, pub kind:EffectKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EffectKind {
    Letterbox{ratio:f32}, ColorGrade{lut:String,strength:f32},
    Vignette{strength:f32}, FilmGrain{intensity:f32},
    SlowMotion{scale:f32,ramp:f32}, Flash{color:[f32;4],dur:f32},
    Desaturate{amount:f32}, Glitch{intensity:f32,freq:f32},
    DepthOfField{focus:String,aperture:f32},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct StateChange { pub t:f32, pub kind:StateChangeKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum StateChangeKind {
    SetFlag{flag:String,val:bool}, StartQuest{id:String}, CompleteQuest{id:String},
    SpawnEntity{id:String,pos:[f32;3]}, DestroyEntity{id:String},
    ChangeWeather{weather:String}, TeleportPlayer{pos:[f32;3]},
    GiveItem{id:String,qty:u32}, PlayCutscene{id:String}, LoadScene{id:String},
}

// ── AI Director ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AiDirector {
    pub enabled:bool, pub style:DirectorStyle,
    pub cuts_per_min:f32, pub music_sync:bool,
    pub close_up_dialogue:bool, pub wide_in_action:bool,
    pub min_shot_secs:f32, pub max_shot_secs:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DirectorStyle { Suspense,Action,Cerebral,Minimalist,Emotional,Realist,Custom(String) }

impl Default for AiDirector {
    fn default()->Self{Self{enabled:true,style:DirectorStyle::Emotional,cuts_per_min:12.0,
        music_sync:true,close_up_dialogue:true,wide_in_action:true,min_shot_secs:1.5,max_shot_secs:8.0}}
}

// ── Poster Generator ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PosterConfig {
    pub game_title:String, pub tagline:Option<String>,
    pub style_prompt:String, pub mood:PosterMood,
    pub brand_color:[f32;4], pub characters:Vec<String>,
    pub formats:Vec<PosterFormat>, pub approved:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PosterMood { Epic,Dark,Horror,Whimsical,Action,Mystery,SciFi,Fantasy,PostApoc,Peaceful }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PosterFormat { BoxArt, HeroBanner, StoreThumbnail, SplashScreen, AppIcon, Wallpaper4K, SocialSquare, VerticalMobile }

// ── Credits ───────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreditsConfig {
    pub opening_logos:Vec<LogoCard>, pub title_card:TitleCard,
    pub sections:Vec<CreditsSection>, pub scroll_speed:f32,
    pub music:String, pub background:CreditsBackground,
    pub dynamic_entries:Vec<DynamicCredit>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct LogoCard { pub company:String, pub logo_asset:String, pub display_secs:f32, pub sound:Option<String> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TitleCard { pub title:String, pub year:u32, pub style:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreditsSection { pub header:String, pub entries:Vec<CreditsEntry> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CreditsEntry { pub role:Option<String>, pub name:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CreditsBackground { Black, Scene{id:String}, Particle{fx:String} }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DynamicCredit { pub condition:String, pub entry:CreditsEntry }

// ── Trailer ───────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TrailerTemplate {
    pub id:String, pub kind:TrailerKind, pub target_secs:f32,
    pub segments:Vec<TrailerSegment>, pub music:String,
    pub beat_synced:bool, pub release_info:String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TrailerKind { Announce,Gameplay,Cinematic,Launch,PatchNotes,Short{platform:String} }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TrailerSegment { pub kind:SegKind, pub dur:f32, pub clips:Vec<String>, pub text:Option<String> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SegKind { Hook,Setup,Rising,Climax,Payoff,CallToAction }

// ── Active Cutscene ──────────────────────────────────────────────
#[derive(Debug,Clone)]
pub struct ActiveCutscene { pub id:String, pub time:f32, pub shot_idx:usize, pub paused:bool, pub started:DateTime<Utc> }

// ── Manager ──────────────────────────────────────────────────────
pub struct CinematicsManager {
    pub cutscenes:HashMap<String,Cutscene>, pub active:Option<ActiveCutscene>,
    pub director:AiDirector, pub poster:Option<PosterConfig>,
    pub credits:Option<CreditsConfig>, pub trailers:Vec<TrailerTemplate>,
    pub playback_speed:f32, pub subtitles_lang:String,
    pub total_watched:u64, pub total_time_secs:f64,
}

impl CinematicsManager {
    pub fn new()->Self{
        Self{cutscenes:HashMap::new(),active:None,director:AiDirector::default(),
             poster:None,credits:None,trailers:Vec::new(),playback_speed:1.0,
             subtitles_lang:"en".to_string(),total_watched:0,total_time_secs:0.0}
    }
    pub fn register(&mut self,c:Cutscene){
        tracing::info!("Registered cutscene: {} ({:.1}s)",c.name,c.duration_secs);
        self.cutscenes.insert(c.id.clone(),c);
    }
    pub fn play(&mut self,id:&str)->bool{
        if self.active.is_some(){return false;}
        if !self.cutscenes.contains_key(id){tracing::error!("Cutscene not found: {}",id);return false;}
        tracing::info!("Playing cutscene: {}",id);
        self.active=Some(ActiveCutscene{id:id.to_string(),time:0.0,shot_idx:0,paused:false,started:Utc::now()});
        true
    }
    pub fn skip(&mut self){
        if let Some(a)=&self.active{
            let skippable=self.cutscenes.get(&a.id).map(|c|c.skippable).unwrap_or(false);
            let skip_after=self.cutscenes.get(&a.id).map(|c|c.skip_after_secs).unwrap_or(0.0);
            if skippable && a.time>=skip_after { self.active=None; self.total_watched+=1; }
        }
    }
    pub fn tick(&mut self,delta:f32){
        if let Some(a)=&mut self.active{
            if a.paused{return;}
            a.time+=delta*self.playback_speed;
            let dur=self.cutscenes.get(&a.id).map(|c|c.duration_secs).unwrap_or(0.0);
            if a.time>=dur{
                self.total_time_secs+=(Utc::now()-a.started).num_seconds() as f64;
                self.total_watched+=1;
                self.active=None;
            }
        }
    }
    pub fn is_playing(&self)->bool{self.active.is_some()}
    pub fn current_time(&self)->f32{self.active.as_ref().map(|a|a.time).unwrap_or(0.0)}
    pub fn count(&self)->usize{self.cutscenes.len()}
    pub fn total_runtime(&self)->f64{self.cutscenes.values().map(|c|c.duration_secs as f64).sum()}
}
extern crate tracing;
