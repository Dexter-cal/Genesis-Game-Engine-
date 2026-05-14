//! Genesis AI Modeler — text-to-3D, retopology, texture gen, LOD gen, mesh repair
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AiModelRequest {
    pub id:String, pub kind:RequestKind, pub status:Status,
    pub priority:u32, pub submitted:DateTime<Utc>,
    pub result:Option<ModelResult>, pub error:Option<String>,
    pub tokens:u32, pub requester:String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum RequestKind {
    TextTo3D{prompt:String,style:String,polys:u32,format:MeshFmt},
    ImageTo3D{path:String,views:u32,format:MeshFmt},
    Retopo{mesh:String,target_polys:u32,flow:EdgeFlow},
    TextureGen{mesh:String,prompt:String,res:u32,maps:Vec<TexMap>},
    AutoRig{mesh:String,char_kind:CharKind},
    LodGen{mesh:String,levels:u32,ratios:Vec<f32>},
    MeshRepair{mesh:String,issues:Vec<MeshIssue>},
    StyleTransfer{src:String,ref_:String,strength:f32},
    UvUnwrap{mesh:String,method:UvMethod},
    Optimize{mesh:String,target:u32,silhouette:bool},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MeshFmt{Glb,Gltf,Obj,Fbx,Ply,Usd}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EdgeFlow{Organic,HardSurface,Character,Creature,Vehicle}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TexMap{Albedo,Normal,Roughness,Metallic,Ao,Emissive,Height,Opacity}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CharKind{Humanoid,Quadruped,Bird,Fish,Custom(String)}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MeshIssue{NonManifold,Holes,InvertedNormals,Duplicates,ZeroFaces,Overlap}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum UvMethod{Smart,Lscm,Conformal,Cubic,Cylindrical,Spherical}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum Status{Queued,Processing{pct:u32},Complete,Failed,Cancelled}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ModelResult {
    Mesh{path:String,polys:u32,verts:u32,bytes:u64},
    Texture{paths:HashMap<String,String>,res:[u32;2]},
    Rig{armature:String,bones:u32},
    Lods{paths:Vec<String>,polys:Vec<u32>},
    Report{fixed:u32,remaining:u32,details:String},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Backend {
    TripoSr{key:String}, ShapE{local:bool}, Meshy{key:String},
    Local{model:String}, Custom{url:String,key:String},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct StylePreset {
    pub id:String, pub name:String, pub prompt_prefix:String,
    pub poly_budget:u32, pub tex_res:u32, pub maps:Vec<TexMap>,
}

pub struct AiModelerAgent {
    pub queue:Vec<AiModelRequest>, pub completed:Vec<AiModelRequest>,
    pub backend:Option<Backend>, pub max_concurrent:u32, pub active:u32,
    pub total_done:u64, pub total_tokens:u64, pub output_dir:String,
    pub styles:HashMap<String,StylePreset>, pub enabled:bool,
}

impl AiModelerAgent {
    pub fn new(out:&str)->Self{
        let mut styles=HashMap::new();
        styles.insert("game_ready".to_string(),StylePreset{
            id:"game_ready".to_string(),name:"Game Ready".to_string(),
            prompt_prefix:"game-ready asset, clean topology, optimised, ".to_string(),
            poly_budget:8000,tex_res:2048,
            maps:vec![TexMap::Albedo,TexMap::Normal,TexMap::Roughness,TexMap::Metallic,TexMap::Ao],
        });
        styles.insert("stylized".to_string(),StylePreset{
            id:"stylized".to_string(),name:"Stylized".to_string(),
            prompt_prefix:"stylized cartoon 3D, vibrant, cel-shaded, ".to_string(),
            poly_budget:4000,tex_res:1024,maps:vec![TexMap::Albedo,TexMap::Emissive],
        });
        styles.insert("realistic".to_string(),StylePreset{
            id:"realistic".to_string(),name:"Photorealistic".to_string(),
            prompt_prefix:"photorealistic 3D, high detail, PBR, ".to_string(),
            poly_budget:50000,tex_res:4096,
            maps:vec![TexMap::Albedo,TexMap::Normal,TexMap::Roughness,TexMap::Metallic,TexMap::Ao,TexMap::Height],
        });
        Self{queue:Vec::new(),completed:Vec::new(),backend:None,
             max_concurrent:2,active:0,total_done:0,total_tokens:0,
             output_dir:out.to_string(),styles,enabled:true}
    }

    pub fn submit(&mut self,kind:RequestKind,requester:&str)->String{
        let id=format!("aim_{}", Utc::now().timestamp_millis());
        tracing::info!("AI Modeler request: {}",&id);
        self.queue.push(AiModelRequest{
            id:id.clone(),kind,status:Status::Queued,priority:5,
            submitted:Utc::now(),result:None,error:None,tokens:0,
            requester:requester.to_string(),
        });
        id
    }

    pub fn cancel(&mut self,id:&str)->bool{
        if let Some(r)=self.queue.iter_mut().find(|r|r.id==id){
            if r.status==Status::Queued{r.status=Status::Cancelled;return true;}
        }
        false
    }

    pub fn status(&self,id:&str)->Option<&Status>{
        self.queue.iter().find(|r|r.id==id)
            .or_else(||self.completed.iter().find(|r|r.id==id))
            .map(|r|&r.status)
    }

    pub fn queue_len(&self)->usize{self.queue.len()}
    pub fn done(&self)->u64{self.total_done}
}
extern crate tracing;
