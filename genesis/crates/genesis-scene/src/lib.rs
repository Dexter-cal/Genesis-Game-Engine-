//! Genesis Scene — scene tree, node graph, serialization, live reload
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Scene{
    pub id:String,pub name:String,pub version:u32,
    pub root:SceneNode,pub metadata:SceneMeta,
    pub lighting:Lighting,pub physics:PhysicsConfig,pub audio:AudioConfig,
    pub modified:DateTime<Utc>,pub created:DateTime<Utc>,pub author:String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SceneNode{
    pub id:String,pub name:String,pub kind:String,
    pub transform:NodeTransform,pub props:HashMap<String,serde_json::Value>,
    pub children:Vec<SceneNode>,pub enabled:bool,pub locked:bool,
    pub scripts:Vec<String>,pub groups:Vec<String>,
}
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct NodeTransform{pub pos:[f32;3],pub rot:[f32;3],pub scale:[f32;3]}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct SceneMeta{pub desc:String,pub spawn_points:Vec<[f32;3]>,pub is_boss:bool,pub is_safe:bool,pub playtime_mins:f32}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Lighting{pub sun_dir:[f32;3],pub sun_color:[f32;4],pub sun_intensity:f32,pub ambient:[f32;4],pub fog:bool,pub fog_color:[f32;4],pub fog_density:f32,pub bloom:bool,pub exposure:f32}
impl Default for Lighting{fn default()->Self{Self{sun_dir:[-0.5,-1.0,-0.5],sun_color:[1.0,0.95,0.8,1.0],sun_intensity:1.0,ambient:[0.3,0.35,0.45,1.0],fog:false,fog_color:[0.8,0.85,0.9,1.0],fog_density:0.01,bloom:true,exposure:1.0}}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PhysicsConfig{pub gravity:[f32;3],pub fps:u32}
impl Default for PhysicsConfig{fn default()->Self{Self{gravity:[0.0,-9.81,0.0],fps:60}}}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioConfig{pub master:f32,pub music:f32,pub sfx:f32,pub voice:f32,pub reverb:String}
impl Default for AudioConfig{fn default()->Self{Self{master:1.0,music:0.8,sfx:1.0,voice:1.0,reverb:"medium_room".to_string()}}}

pub struct SceneManager{
    pub scenes:HashMap<String,Scene>,pub active:Option<String>,
    pub history:Vec<String>,pub loading:bool,pub total:u64,
}
impl SceneManager{
    pub fn new()->Self{Self{scenes:HashMap::new(),active:None,history:Vec::new(),loading:false,total:0}}
    pub fn register(&mut self,s:Scene){
        tracing::info!("Registered scene: {} ({})",s.name,s.id);
        self.scenes.insert(s.id.clone(),s);
    }
    pub fn load(&mut self,id:&str)->bool{
        if !self.scenes.contains_key(id){return false;}
        if let Some(prev)=self.active.take(){self.history.push(prev);}
        self.active=Some(id.to_string());
        self.loading=true;self.total+=1;
        tracing::info!("Loading scene: {}",id);
        true
    }
    pub fn go_back(&mut self)->bool{
        self.history.pop().map(|id|self.load(&id)).unwrap_or(false)
    }
    pub fn active(&self)->Option<&Scene>{self.active.as_ref().and_then(|id|self.scenes.get(id))}
    pub fn node_count(&self)->usize{self.active().map(|s|count(&s.root)).unwrap_or(0)}
    pub fn count(&self)->usize{self.scenes.len()}
}
fn count(n:&SceneNode)->usize{1+n.children.iter().map(count).sum::<usize>()}
extern crate tracing;
