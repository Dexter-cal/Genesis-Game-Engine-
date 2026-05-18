//! Editor — in-engine IDE, property panels, undo/redo, visual debugger, gizmos
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,VecDeque};
use chrono::{DateTime,Utc};

// ── Selection ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Selection {
    pub entities:  Vec<String>,
    pub primary:   Option<String>,
    pub pivot:     [f32;3],
    pub pivot_mode:PivotMode,
}
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub enum PivotMode { #[default] Individual, Median, World, Cursor }
impl Selection {
    pub fn select(&mut self, id:&str) { if !self.entities.contains(&id.to_string()) { self.entities.push(id.to_string()); self.primary=Some(id.to_string()); } }
    pub fn deselect(&mut self,id:&str){ self.entities.retain(|e|e!=id); if self.primary.as_deref()==Some(id){self.primary=self.entities.last().cloned();} }
    pub fn clear(&mut self){self.entities.clear();self.primary=None;}
    pub fn is_selected(&self,id:&str)->bool{self.entities.iter().any(|e|e==id)}
    pub fn count(&self)->usize{self.entities.len()}
}

// ── Undo/Redo ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UndoAction {
    pub id:          String,
    pub description: String,
    pub before:      serde_json::Value,
    pub after:       serde_json::Value,
    pub objects:     Vec<String>,
    pub ts:          DateTime<Utc>,
    pub cost_ms:     f32,
}

pub struct UndoStack {
    pub history:   VecDeque<UndoAction>,
    pub future:    VecDeque<UndoAction>,
    pub max_depth: usize,
    pub total_undos: u64,
    pub total_redos: u64,
}

impl UndoStack {
    pub fn new(max_depth:usize) -> Self {
        Self { history:VecDeque::new(), future:VecDeque::new(), max_depth, total_undos:0, total_redos:0 }
    }
    pub fn push(&mut self, action:UndoAction) {
        self.future.clear();
        self.history.push_back(action);
        while self.history.len() > self.max_depth { self.history.pop_front(); }
    }
    pub fn undo(&mut self) -> Option<&UndoAction> {
        if let Some(a) = self.history.pop_back() {
            self.total_undos += 1;
            tracing::debug!("Undo: {}", a.description);
            self.future.push_front(a);
            self.future.front()
        } else { None }
    }
    pub fn redo(&mut self) -> Option<&UndoAction> {
        if let Some(a) = self.future.pop_front() {
            self.total_redos += 1;
            tracing::debug!("Redo: {}", a.description);
            self.history.push_back(a);
            self.history.back()
        } else { None }
    }
    pub fn can_undo(&self)->bool{!self.history.is_empty()}
    pub fn can_redo(&self)->bool{!self.future.is_empty()}
    pub fn description(&self)->&str{self.history.back().map(|a|a.description.as_str()).unwrap_or("")}
}

// ── Visual Debugger ───────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugDraw {
    pub lines:    Vec<DebugLine>,
    pub boxes:    Vec<DebugBox>,
    pub spheres:  Vec<DebugSphere>,
    pub texts:    Vec<DebugText>,
    pub arrows:   Vec<DebugArrow>,
    pub enabled:  bool,
    pub physics:  bool,
    pub navmesh:  bool,
    pub ai_paths: bool,
    pub bounds:   bool,
    pub sockets:  bool,
    pub normals:  bool,
    pub wireframe:bool,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugLine{pub a:[f32;3],pub b:[f32;3],pub color:[f32;4],pub duration:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugBox{pub center:[f32;3],pub half_ext:[f32;3],pub color:[f32;4],pub duration:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugSphere{pub center:[f32;3],pub radius:f32,pub color:[f32;4],pub duration:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugText{pub pos:[f32;3],pub text:String,pub color:[f32;4],pub duration:f32,pub size:f32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DebugArrow{pub origin:[f32;3],pub dir:[f32;3],pub color:[f32;4],pub duration:f32}

impl DebugDraw {
    pub fn new()->Self{Self{lines:Vec::new(),boxes:Vec::new(),spheres:Vec::new(),texts:Vec::new(),arrows:Vec::new(),enabled:true,physics:false,navmesh:false,ai_paths:false,bounds:false,sockets:false,normals:false,wireframe:false}}
    pub fn line(&mut self,a:[f32;3],b:[f32;3],color:[f32;4],dur:f32){if self.enabled{self.lines.push(DebugLine{a,b,color,duration:dur});}}
    pub fn sphere(&mut self,c:[f32;3],r:f32,color:[f32;4],dur:f32){if self.enabled{self.spheres.push(DebugSphere{center:c,radius:r,color,duration:dur});}}
    pub fn text(&mut self,pos:[f32;3],text:&str,color:[f32;4],dur:f32){if self.enabled{self.texts.push(DebugText{pos,text:text.to_string(),color,duration:dur,size:1.0});}}
    pub fn tick(&mut self,delta:f32){
        for v in [&mut self.lines as &mut Vec<DebugLine>]{v.retain(|l|{l.duration-=delta;l.duration>0.0});}
        self.boxes.retain(|b|{let mut b=b.clone();b.duration-=delta;b.duration>0.0});
        self.spheres.retain(|s|{let mut s=s.clone();s.duration-=delta;s.duration>0.0});
        self.texts.retain(|t|{let mut t=t.clone();t.duration-=delta;t.duration>0.0});
    }
}

// ── Gizmos ────────────────────────────────────────────────────────
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum GizmoMode { Select, Translate, Rotate, Scale, Universal }
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum GizmoSpace { World, Local, Parent, View }

// ── Grid / Snap ───────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EditorGrid {
    pub enabled:     bool,
    pub spacing:     f32,
    pub subdivisions:u32,
    pub opacity:     f32,
    pub color:       [f32;4],
    pub infinite:    bool,
    pub snap_translate:f32,
    pub snap_rotate: f32,
    pub snap_scale:  f32,
    pub snap_enabled:bool,
}
impl Default for EditorGrid {
    fn default()->Self{Self{enabled:true,spacing:1.0,subdivisions:5,opacity:0.3,color:[0.4,0.45,0.55,1.0],infinite:true,snap_translate:0.25,snap_rotate:15.0,snap_scale:0.1,snap_enabled:false}}
}

// ── Editor State ──────────────────────────────────────────────────
pub struct EditorState {
    pub selection:    Selection,
    pub undo:         UndoStack,
    pub debug:        DebugDraw,
    pub gizmo_mode:   GizmoMode,
    pub gizmo_space:  GizmoSpace,
    pub grid:         EditorGrid,
    pub play_mode:    PlayMode,
    pub cam_pos:      [f32;3],
    pub cam_rot:      [f32;3],
    pub cam_fov:      f32,
    pub ortho:        bool,
    pub ortho_scale:  f32,
    pub viewport_shading: ViewportShading,
    pub stats_overlay:    bool,
    pub ai_panel_open:    bool,
    pub properties_open:  bool,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PlayMode { Edit, Playing, Paused }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ViewportShading { Solid, Material, Rendered, Wireframe, Xray }

impl EditorState {
    pub fn new()->Self{
        Self{
            selection:Selection::default(), undo:UndoStack::new(200),
            debug:DebugDraw::new(), gizmo_mode:GizmoMode::Select,
            gizmo_space:GizmoSpace::World, grid:EditorGrid::default(),
            play_mode:PlayMode::Edit, cam_pos:[0.0,5.0,10.0],
            cam_rot:[-15.0,0.0,0.0], cam_fov:60.0, ortho:false,
            ortho_scale:10.0, viewport_shading:ViewportShading::Solid,
            stats_overlay:true, ai_panel_open:true, properties_open:true,
        }
    }
    pub fn is_playing(&self)->bool{matches!(self.play_mode,PlayMode::Playing)}
    pub fn toggle_play(&mut self){
        self.play_mode = match self.play_mode {
            PlayMode::Edit    => { tracing::info!("Editor → Play"); PlayMode::Playing },
            PlayMode::Playing => { tracing::info!("Play → Pause");  PlayMode::Paused },
            PlayMode::Paused  => { tracing::info!("Pause → Play");  PlayMode::Playing },
        };
    }
    pub fn stop(&mut self){ self.play_mode=PlayMode::Edit; tracing::info!("Stopped → Edit mode"); }
    pub fn tick(&mut self,delta:f32){ self.debug.tick(delta); }
}
extern crate tracing;
