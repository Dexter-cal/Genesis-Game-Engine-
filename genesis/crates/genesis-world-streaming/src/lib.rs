//! Genesis World Streaming — open world chunks, biomes, weather, time, LOD
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,BinaryHeap};
use std::cmp::Ordering;
use chrono::{DateTime,Utc};

// ── Chunk Coords ─────────────────────────────────────────────────
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub struct ChunkCoord { pub x:i32, pub z:i32 }
impl ChunkCoord {
    pub fn new(x:i32,z:i32)->Self{Self{x,z}}
    pub fn dist_sq(&self,o:&Self)->i32{let dx=self.x-o.x;let dz=self.z-o.z;dx*dx+dz*dz}
    pub fn world_center(&self,size:f32)->[f32;3]{[self.x as f32*size+size*0.5,0.0,self.z as f32*size+size*0.5]}
    pub fn from_world(x:f32,z:f32,size:f32)->Self{Self{x:(x/size).floor() as i32,z:(z/size).floor() as i32}}
    pub fn neighbors(&self)->Vec<Self>{(-1i32..=1).flat_map(|dx|(-1i32..=1).map(move|dz|Self{x:self.x+dx,z:self.z+dz})).filter(|c|c!=self).collect()}
}
impl std::fmt::Display for ChunkCoord{fn fmt(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{write!(f,"Chunk({},{})",self.x,self.z)}}

// ── Chunk ────────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WorldChunk {
    pub coord:ChunkCoord, pub state:ChunkState,
    pub heightmap:Vec<f32>, pub resolution:u32,
    pub layer_weights:Vec<Vec<f32>>,
    pub objects:Vec<ChunkObject>, pub spawners:Vec<SpawnerData>,
    pub lights:Vec<ChunkLight>, pub audio_zones:Vec<AudioZone>,
    pub triggers:Vec<TriggerVolume>, pub biome_id:String,
    pub player_modified:bool, pub navmesh_dirty:bool,
    pub version:u32, pub memory_bytes:u64,
    pub last_visited:Option<DateTime<Utc>>,
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum ChunkState { Unloaded, Queued{priority:i32}, Loading, Generating, Loaded, Active, Saving, Error(String) }

impl WorldChunk {
    pub fn height_at(&self,lx:f32,lz:f32)->f32 {
        let r=(self.resolution-1) as f32;
        let xi=(lx*r).floor() as usize; let zi=(lz*r).floor() as usize;
        let xi=xi.min(self.resolution as usize-2); let zi=zi.min(self.resolution as usize-2);
        let fx=lx*r-xi as f32; let fz=lz*r-zi as f32;
        let rs=self.resolution as usize;
        let h00=self.heightmap[zi*rs+xi]; let h10=self.heightmap[zi*rs+xi+1];
        let h01=self.heightmap[(zi+1)*rs+xi]; let h11=self.heightmap[(zi+1)*rs+xi+1];
        h00*(1.0-fx)*(1.0-fz)+h10*fx*(1.0-fz)+h01*(1.0-fx)*fz+h11*fx*fz
    }
    pub fn deform(&mut self,lx:f32,lz:f32,radius_cells:usize,amount:f32) {
        let r=self.resolution as usize;
        let cx=(lx*(r-1) as f32) as usize; let cz=(lz*(r-1) as f32) as usize;
        for dz in 0..=radius_cells*2 { for dx in 0..=radius_cells*2 {
            let ix=cx.saturating_sub(radius_cells)+dx;
            let iz=cz.saturating_sub(radius_cells)+dz;
            if ix>=r||iz>=r { continue; }
            let ddx=ix as f32-cx as f32; let ddz=iz as f32-cz as f32;
            let dist=(ddx*ddx+ddz*ddz).sqrt()/radius_cells as f32;
            if dist>1.0 { continue; }
            self.heightmap[iz*r+ix]-=amount*(1.0-dist);
        }}
        self.player_modified=true;
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChunkObject { pub id:String, pub prefab:String, pub pos:[f32;3], pub rot:[f32;4], pub scale:[f32;3], pub destroyed:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SpawnerData { pub id:String, pub creature:String, pub max:u32, pub current:u32, pub respawn_secs:f32, pub active:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ChunkLight { pub pos:[f32;3], pub color:[f32;3], pub intensity:f32, pub range:f32, pub kind:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AudioZone { pub pos:[f32;3], pub radius:f32, pub clip:String, pub volume:f32, pub loop_:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TriggerVolume { pub id:String, pub pos:[f32;3], pub radius:f32, pub event_enter:Option<String>, pub event_exit:Option<String>, pub one_shot:bool, pub triggered:bool }

// ── Biomes ───────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BiomeDef {
    pub id:String, pub name:String, pub kind:BiomeKind,
    pub temp_range:[f32;2], pub moisture_range:[f32;2], pub altitude_range:[f32;2],
    pub color:[f32;4], pub fog_density:f32, pub vegetation_density:f32,
    pub tree_coverage:f32, pub wildlife:Vec<String>, pub ambient_sounds:Vec<String>,
    pub music:Option<String>, pub ground_material:String,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BiomeKind {
    Tundra,Taiga,TemperateForest,Grassland,Savanna,Desert,TropicalRainforest,
    Ocean{depth:f32},Wetland,Swamp,Volcanic,Arctic,Alpine,Canyon,
    Crystal{color:[f32;4]},Corrupted{level:f32},Ethereal,Infernal,
    Shadow{darkness:f32},Arcane,Fungal,Custom(String),
}

// ── Weather ──────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WeatherState {
    pub id:String, pub name:String, pub cloud_coverage:f32, pub cloud_type:String,
    pub precipitation:PrecipKind, pub precip_intensity:f32,
    pub wind_speed:f32, pub wind_dir:f32, pub visibility_m:f32,
    pub lightning:bool, pub fog_density:f32, pub sun_intensity:f32,
    pub sky_color:[f32;4], pub weight:f32, pub audio:Vec<String>,
    pub gameplay:Vec<WeatherEffect>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PrecipKind { None, Rain, HeavyRain, Snow, Hail, Sleet, Ash, MagicDust }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum WeatherEffect { MovementMult{v:f32}, VisibilityMult{v:f32}, SlipperyGround{f:f32}, FireDmgBonus{v:f32}, ColdDmgBonus{v:f32}, StaminaDrain{r:f32} }

// ── Load Request (priority queue) ────────────────────────────────
#[derive(Debug,Eq,PartialEq)]
pub struct LoadReq { pub coord:ChunkCoord, pub priority:i32 }
impl Ord for LoadReq { fn cmp(&self,o:&Self)->Ordering{self.priority.cmp(&o.priority)} }
impl PartialOrd for LoadReq { fn partial_cmp(&self,o:&Self)->Option<Ordering>{Some(self.cmp(o))} }

// ── World Streaming Manager ──────────────────────────────────────
pub struct WorldStreamingManager {
    pub chunks:HashMap<ChunkCoord,WorldChunk>,
    pub load_queue:BinaryHeap<LoadReq>,
    pub unload_queue:Vec<ChunkCoord>,
    pub biomes:HashMap<String,BiomeDef>,
    pub weather_states:HashMap<String,WeatherState>,
    pub current_weather:String,
    pub weather_transition:Option<(String,f32,f32)>, // to, duration, elapsed
    pub time_of_day:f32, pub day:u32, pub season:Season,
    pub day_duration_secs:f32, pub sunrise_hour:f32, pub sunset_hour:f32,
    pub player_chunk:ChunkCoord, pub player_pos:[f32;3],
    pub chunk_size_m:f32, pub height_scale:f32,
    pub load_radius:u32, pub unload_radius:u32,
    pub max_memory_mb:u64, pub memory_used_bytes:u64,
    pub total_loaded:u64, pub total_generated:u64,
    pub wind_speed:f32, pub wind_dir:f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Season { Spring, Summer, Autumn, Winter }

impl WorldStreamingManager {
    pub fn new(chunk_size_m:f32, height_scale:f32) -> Self {
        Self {
            chunks:HashMap::new(), load_queue:BinaryHeap::new(),
            unload_queue:Vec::new(), biomes:HashMap::new(),
            weather_states:HashMap::new(), current_weather:"Clear".to_string(),
            weather_transition:None, time_of_day:8.0, day:1, season:Season::Summer,
            day_duration_secs:1200.0, sunrise_hour:6.0, sunset_hour:20.0,
            player_chunk:ChunkCoord::new(0,0), player_pos:[0.0;3],
            chunk_size_m, height_scale,
            load_radius:5, unload_radius:8,
            max_memory_mb:2048, memory_used_bytes:0,
            total_loaded:0, total_generated:0,
            wind_speed:3.0, wind_dir:45.0,
        }
    }

    pub fn update_player(&mut self, pos:[f32;3]) {
        self.player_pos=pos;
        let new_chunk=ChunkCoord::from_world(pos[0],pos[2],self.chunk_size_m);
        if new_chunk!=self.player_chunk {
            self.player_chunk=new_chunk;
            self.rebuild_queue();
        }
    }

    fn rebuild_queue(&mut self) {
        self.load_queue.clear();
        let r=self.load_radius as i32;
        let pc=self.player_chunk;
        for dx in -r..=r { for dz in -r..=r {
            let c=ChunkCoord::new(pc.x+dx,pc.z+dz);
            let dsq=c.dist_sq(&pc);
            if dsq<=r*r && !self.chunks.contains_key(&c) {
                self.load_queue.push(LoadReq{coord:c,priority:-(dsq)});
            }
        }}
        let ur=self.unload_radius as i32;
        self.unload_queue=self.chunks.keys().filter(|c|c.dist_sq(&pc)>ur*ur).cloned().collect();
    }

    pub fn tick(&mut self, delta:f32, budget_ms:f32) {
        // Advance time
        self.time_of_day += delta/self.day_duration_secs*24.0;
        if self.time_of_day>=24.0 { self.time_of_day-=24.0; self.day+=1; }
        // Weather transition
        if let Some((to,dur,ref mut elapsed)) = &mut self.weather_transition {
            *elapsed+=delta;
            if *elapsed>=*dur { let t=to.clone(); self.current_weather=t; self.weather_transition=None; }
        }
        // Load chunks
        let start=std::time::Instant::now();
        while start.elapsed().as_millis()<budget_ms as u128 {
            if let Some(req)=self.load_queue.pop() {
                if !self.chunks.contains_key(&req.coord) { self.load_chunk(req.coord); }
            } else { break; }
        }
        // Unload
        for coord in self.unload_queue.drain(..self.unload_queue.len().min(4)) {
            self.unload_chunk(coord);
        }
    }

    fn load_chunk(&mut self, coord:ChunkCoord) {
        let res=65u32;
        let mut hm=vec![0.0f32;(res*res) as usize];
        // Simple procedural height using pseudo-noise
        for z in 0..res { for x in 0..res {
            let wx=coord.x as f32*self.chunk_size_m+x as f32*(self.chunk_size_m/(res-1) as f32);
            let wz=coord.z as f32*self.chunk_size_m+z as f32*(self.chunk_size_m/(res-1) as f32);
            hm[(z*res+x) as usize]=(wx*0.01).sin()*(wz*0.01).cos()*10.0+5.0;
        }}
        let chunk=WorldChunk {
            coord, state:ChunkState::Loaded, heightmap:hm, resolution:res,
            layer_weights:Vec::new(), objects:Vec::new(), spawners:Vec::new(),
            lights:Vec::new(), audio_zones:Vec::new(), triggers:Vec::new(),
            biome_id:"grassland".to_string(), player_modified:false,
            navmesh_dirty:true, version:1, memory_bytes:512*1024,
            last_visited:Some(Utc::now()),
        };
        self.memory_used_bytes+=chunk.memory_bytes;
        self.total_loaded+=1;
        self.chunks.insert(coord,chunk);
    }

    fn unload_chunk(&mut self, coord:ChunkCoord) {
        if let Some(c)=self.chunks.remove(&coord) {
            self.memory_used_bytes=self.memory_used_bytes.saturating_sub(c.memory_bytes);
        }
    }

    pub fn set_weather(&mut self, id:&str, transition_secs:f32) {
        tracing::info!("Weather → {} over {:.0}s",id,transition_secs);
        self.weather_transition=Some((id.to_string(),transition_secs,0.0));
    }
    pub fn height_at(&self,x:f32,z:f32)->f32 {
        let coord=ChunkCoord::from_world(x,z,self.chunk_size_m);
        let lx=(x-coord.x as f32*self.chunk_size_m)/self.chunk_size_m;
        let lz=(z-coord.z as f32*self.chunk_size_m)/self.chunk_size_m;
        self.chunks.get(&coord).map(|c|c.height_at(lx.clamp(0.0,1.0),lz.clamp(0.0,1.0))*self.height_scale).unwrap_or(0.0)
    }
    pub fn is_daytime(&self)->bool { self.time_of_day>=self.sunrise_hour && self.time_of_day<self.sunset_hour }
    pub fn loaded_count(&self)->usize{self.chunks.len()}
    pub fn memory_mb(&self)->f64{self.memory_used_bytes as f64/1_048_576.0}
    pub fn register_biome(&mut self,b:BiomeDef){self.biomes.insert(b.id.clone(),b);}
}
extern crate tracing;
