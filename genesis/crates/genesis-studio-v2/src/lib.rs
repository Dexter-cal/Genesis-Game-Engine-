//! Genesis Studio V2 — AI-powered 3D modeler, production studio, VFX compositor, audio suite
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Modeler Studio ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ModelerStudio {
    pub objects:Vec<ModelObject>, pub selected:Vec<String>,
    pub edit_mode:EditMode, pub brush:SculptBrush,
    pub symmetry:Symmetry, pub snap:SnapState,
    pub timeline:AnimTimeline, pub undo_count:usize,
    pub ai_suggestions:Vec<AiSuggestion>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ModelObject {
    pub id:String, pub name:String, pub kind:ObjKind,
    pub transform:Transform3D, pub modifiers:Vec<Modifier>,
    pub materials:Vec<String>, pub shape_keys:Vec<ShapeKey>,
    pub vertex_groups:Vec<String>, pub visible:bool, pub locked:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ObjKind { Mesh{verts:u32,polys:u32}, Curve, Armature{bones:u32}, Light, Camera, Empty, Volume }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Transform3D{pub pos:[f32;3],pub rot:[f32;3],pub scale:[f32;3]}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ShapeKey{pub name:String,pub value:f32,pub basis:bool}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EditMode { Object, Edit{select:SelectMode}, Sculpt{dyntopo:bool}, WeightPaint, TexturePaint, UV }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SelectMode { Vertex, Edge, Face }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SculptBrush {
    pub kind:BrushKind, pub radius:f32, pub strength:f32,
    pub hardness:f32, pub auto_smooth:f32, pub direction:bool,
    pub symmetry_x:bool, pub symmetry_y:bool, pub symmetry_z:bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BrushKind {
    Draw,DrawSharp,Clay,ClayStrips,ClayThumb,Layer,Inflate,Blob,Crease,
    Smooth,Flatten,Fill,Scrape,Pinch,Grab,Snake,Pose,Thumb,Boundary,
    Cloth,Simplify,Mask,
    AiRefine, AiSculpt, AiSmooth{preserve:bool}, AiRetopo{flow:String}, AiTexture,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Symmetry{pub x:bool,pub y:bool,pub z:bool,pub radial_x:u32,pub radial_y:u32,pub radial_z:u32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SnapState{pub enabled:bool,pub to:Vec<String>,pub align_rotation:bool}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Modifier{pub id:String,pub name:String,pub kind:String,pub enabled:bool,pub props:serde_json::Value}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AnimTimeline {
    pub start:i32,pub end:i32,pub current:i32,pub fps:f32,
    pub playing:bool,pub loop_:bool,pub channels:Vec<AnimChannel>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AnimChannel{pub object:String,pub prop:String,pub keyframes:Vec<Keyframe>,pub muted:bool}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Keyframe{pub frame:i32,pub value:f32,pub ease:String}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AiSuggestion{pub desc:String,pub action:String,pub confidence:f32,pub preview:Option<String>}

// ── Production Studio ─────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProductionStudio {
    pub session:Session, pub actors:Vec<Actor>,
    pub virtual_cams:Vec<VirtualCamera>, pub devices:Vec<CaptureDevice>,
    pub timeline:ProductionTimeline, pub render_mode:String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Session {
    pub id:String, pub name:String, pub status:SessionStatus,
    pub started:Option<DateTime<Utc>>, pub takes:Vec<Take>,
    pub current_take:u32, pub participants:Vec<String>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SessionStatus{Idle,Rehearsal,Recording,Reviewing,Editing,Exporting}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Take{pub n:u32,pub name:String,pub dur_ms:u64,pub rating:u8,pub approved:bool,pub notes:String}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Actor{
    pub id:String,pub name:String,pub character:Option<String>,
    pub device:Option<String>,pub face_tracking:bool,pub body_tracking:bool,
    pub face_clone:Option<String>,pub voice_clone:Option<String>,pub tracking_quality:f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VirtualCamera{
    pub id:String,pub name:String,pub pos:[f32;3],pub rot:[f32;3],
    pub focal_mm:f32,pub aperture:f32,pub focus_dist:f32,pub dof:bool,
    pub mode:CamMode,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CamMode{Handheld{shake:f32},Crane{smooth:f32},Steadicam,Tripod,Dolly,Drone{stab:f32},Oner}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CaptureDevice{
    pub id:String,pub name:String,pub kind:DeviceKind,
    pub res:[u32;2],pub fps:f32,pub connected:bool,pub latency_ms:u32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DeviceKind{Phone{qr:String},Webcam{path:String},Depth{model:String},Dslr{url:String},Ip{url:String}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProductionTimeline{
    pub dur_secs:f32,pub current:f32,pub fps:f32,pub playing:bool,
    pub tracks:Vec<ProdTrack>,pub markers:Vec<(f32,String)>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProdTrack{
    pub id:String,pub name:String,pub kind:TrackKind,
    pub clips:Vec<ProdClip>,pub muted:bool,pub locked:bool,pub volume:f32,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TrackKind{Video,Audio,CharAnim{char:String},CameraAnim,VfxLayer,Subtitle,Music,Ambient}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProdClip{pub id:String,pub start:f32,pub dur:f32,pub path:Option<String>,pub speed:f32,pub opacity:f32}

// ── VFX Compositor ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VfxCompositor{
    pub nodes:HashMap<String,CompNode>,pub connections:Vec<CompConn>,
    pub output:Option<String>,pub resolution:[u32;2],pub fps:f32,
    pub color_space:String,pub gpu:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CompNode{pub id:String,pub label:String,pub kind:CompNodeKind,pub pos:[f32;2],pub muted:bool,pub props:serde_json::Value}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CompConn{pub from:String,pub from_sock:String,pub to:String,pub to_sock:String}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CompNodeKind {
    MediaIn,LiveCapture,GameRender,Image,Movie,Mask,
    Composite,Viewer,FileOutput,
    ColorBalance,Gamma,Exposure,Tonemap,HueSat,BrightContrast,Invert,Mix,
    ChromaKey,LumaKey,ColorKey,DiffKey,
    Blur,Sharpen,Denoise,Defocus,BokehBlur,
    Transform,Crop,Flip,Scale,Rotate,
    Glow,Vignette,Lens,LensDistort,FilmGrain,
    AiFaceReplace,AiBodyReplace,AiRotoscope{quality:u32},
    AiUpscale{scale:f32},AiColorGrade{style:String},AiStabilize,
    AiDeNoise{strength:f32},AiSkyReplace{prompt:String},AiObjectRemove,
    ZCombine,DepthOfField,Split,Merge,Switch,Group,
}

// ── Audio Suite ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioSuite{
    pub tracks:Vec<AudioTrack>,pub master:MasterBus,
    pub sample_rate:u32,pub bit_depth:u32,pub buffer_size:u32,
    pub plugins:Vec<AudioPlugin>,pub spatial:SpatialConfig,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioTrack{
    pub id:String,pub name:String,pub kind:String,
    pub volume:f32,pub pan:f32,pub muted:bool,pub solo:bool,pub armed:bool,
    pub clips:Vec<AudioClip>,pub fx:Vec<AudioFx>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioClip{pub id:String,pub path:String,pub start:f32,pub len:f32,pub gain:f32,pub fade_in:f32,pub fade_out:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioFx{pub name:String,pub kind:FxKind,pub enabled:bool,pub params:HashMap<String,f32>}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum FxKind{Eq{bands:u32},Compressor,Limiter,Reverb,Delay,Chorus,Flanger,Gate,AiDeNoise,AiVoiceClone{voice:String},AiMusicSep}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MasterBus{pub volume:f32,pub fx:Vec<AudioFx>,pub metering:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioPlugin{pub id:String,pub name:String,pub format:String,pub vendor:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SpatialConfig{pub engine:String,pub hrtf:bool,pub room_sim:bool,pub listener:Option<String>}

// ── Studio Manager ────────────────────────────────────────────────
pub struct StudioManager {
    pub modeler:ModelerStudio,
    pub production:ProductionStudio,
    pub compositor:VfxCompositor,
    pub audio:AudioSuite,
    pub active_mode:StudioMode,
}

#[derive(Debug,Clone,PartialEq)]
pub enum StudioMode{Modeler,Production,Compositor,Audio}

impl StudioManager {
    pub fn new()->Self{
        Self{
            modeler:ModelerStudio{
                objects:Vec::new(),selected:Vec::new(),
                edit_mode:EditMode::Object,
                brush:SculptBrush{kind:BrushKind::Draw,radius:50.0,strength:0.5,hardness:0.5,auto_smooth:0.0,direction:true,symmetry_x:false,symmetry_y:false,symmetry_z:false},
                symmetry:Symmetry{x:false,y:false,z:false,radial_x:1,radial_y:1,radial_z:1},
                snap:SnapState{enabled:false,to:Vec::new(),align_rotation:false},
                timeline:AnimTimeline{start:1,end:250,current:1,fps:24.0,playing:false,loop_:false,channels:Vec::new()},
                undo_count:0,ai_suggestions:Vec::new(),
            },
            production:ProductionStudio{
                session:Session{id:String::new(),name:"New Session".to_string(),
                    status:SessionStatus::Idle,started:None,takes:Vec::new(),current_take:1,participants:Vec::new()},
                actors:Vec::new(),virtual_cams:Vec::new(),devices:Vec::new(),
                timeline:ProductionTimeline{dur_secs:120.0,current:0.0,fps:24.0,playing:false,tracks:Vec::new(),markers:Vec::new()},
                render_mode:"realtime".to_string(),
            },
            compositor:VfxCompositor{
                nodes:HashMap::new(),connections:Vec::new(),output:None,
                resolution:[1920,1080],fps:24.0,color_space:"ACES".to_string(),gpu:true,
            },
            audio:AudioSuite{
                tracks:Vec::new(),
                master:MasterBus{volume:1.0,fx:Vec::new(),metering:"LUFS".to_string()},
                sample_rate:48000,bit_depth:24,buffer_size:256,
                plugins:Vec::new(),
                spatial:SpatialConfig{engine:"GenesisNative".to_string(),hrtf:true,room_sim:true,listener:None},
            },
            active_mode:StudioMode::Modeler,
        }
    }

    pub fn switch_mode(&mut self,mode:StudioMode){
        tracing::info!("Studio switching to {:?}",&mode);
        self.active_mode=mode;
    }

    pub fn add_object(&mut self,obj:ModelObject){
        tracing::info!("Added object: {}",obj.name);
        self.modeler.objects.push(obj);
    }

    pub fn start_recording(&mut self)->bool{
        if matches!(self.production.session.status,SessionStatus::Recording){return false;}
        self.production.session.status=SessionStatus::Recording;
        self.production.session.started=Some(Utc::now());
        tracing::info!("Production recording started");
        true
    }

    pub fn add_comp_node(&mut self,node:CompNode){
        self.compositor.nodes.insert(node.id.clone(),node);
    }

    pub fn add_audio_track(&mut self,track:AudioTrack){
        self.audio.tracks.push(track);
    }

    pub fn object_count(&self)->usize{self.modeler.objects.len()}
    pub fn track_count(&self)->usize{self.audio.tracks.len()}
    pub fn comp_node_count(&self)->usize{self.compositor.nodes.len()}
    pub fn take_count(&self)->usize{self.production.session.takes.len()}
}
extern crate tracing;
