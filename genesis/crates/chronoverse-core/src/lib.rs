//! GENESIS Engine Core — lifecycle, event bus, plugin trait, entity DNA, frame timing
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ ENGINE STATE ════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum EngineState {
    Uninitialized,
    Booting    { step:String, progress:f32 },
    Running    { frame:u64, fps:f32 },
    Paused     { reason:PauseReason },
    Loading    { scene:String, progress:f32 },
    Exporting  { target:String, progress:f32 },
    ShuttingDown,
    Crashed    { error:String },
}
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum PauseReason { User, FocusLost, Debugging, Modal, Loading }
impl EngineState {
    pub fn is_running(&self) ->bool{matches!(self,Self::Running{..})}
    pub fn is_paused(&self)  ->bool{matches!(self,Self::Paused{..})}
    pub fn is_loading(&self) ->bool{matches!(self,Self::Loading{..}|Self::Booting{..})}
    pub fn frame(&self)      ->u64{if let Self::Running{frame,..}=self{*frame}else{0}}
}

// ═══ EVENT BUS ════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum GameEvent {
    EngineReady,
    FrameStart{frame:u64,delta:f32}, FrameEnd{frame:u64,dt_ms:f32}, Shutdown,
    SceneLoad{id:String}, SceneLoaded{id:String}, SceneUnload{id:String},
    EntitySpawned{id:String,arch:String}, EntityDestroyed{id:String},
    PlayerSpawned{id:String,pos:[f32;3]}, PlayerDied{id:String,cause:String,pos:[f32;3]},
    PlayerRespawned{id:String,pos:[f32;3]}, PlayerLevelUp{id:String,level:u32},
    PlayerAchievement{id:String,achievement:String},
    DamageDealt{src:String,tgt:String,amount:f32,kind:String},
    EntityKilled{killer:String,victim:String,pos:[f32;3]},
    KillStreak{player:String,count:u32},
    ItemPickup{entity:String,item:String,pos:[f32;3]},
    ItemDrop{entity:String,item:String,pos:[f32;3]},
    ItemUsed{entity:String,item:String},
    QuestStarted{quest:String,player:String}, QuestComplete{quest:String,player:String},
    QuestFailed{quest:String,player:String,reason:String},
    ObjectiveComplete{quest:String,obj:String},
    DialogueStart{npc:String,player:String}, DialogueEnd{npc:String,player:String},
    NpcStateChange{npc:String,from:String,to:String},
    WeatherChange{from:String,to:String}, TimeOfDay{hour:f32},
    SeasonChange{season:String}, ZoneEntered{entity:String,zone:String},
    ZoneExited{entity:String,zone:String},
    Collision{a:String,b:String,point:[f32;3],force:f32},
    TriggerEnter{entity:String,trigger:String}, TriggerExit{entity:String,trigger:String},
    Explosion{pos:[f32;3],radius:f32,force:f32,src:String},
    MenuOpen{id:String}, MenuClose{id:String}, ButtonPressed{id:String,src:String},
    SoundPlay{clip:String,pos:Option<[f32;3]>}, MusicChange{track:String,fade:f32},
    PlayerConnected{id:String,session:String}, PlayerDisconnected{id:String,reason:String},
    MatchStart{id:String,players:Vec<String>}, MatchEnd{id:String,winner:Option<String>},
    SaveRequested, SaveComplete{slot:u32}, LoadRequested{slot:u32},
    AchievementUnlocked{player:String,achievement:String},
    Custom{kind:String,data:serde_json::Value},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AgentEvent {
    TaskAssigned{agent:String,task:String,desc:String},
    TaskStarted{agent:String,task:String},
    TaskProgress{agent:String,task:String,pct:f32,status:String},
    TaskComplete{agent:String,task:String,result:serde_json::Value},
    TaskFailed{agent:String,task:String,error:String},
    TaskCancelled{agent:String,task:String},
    AgentThought{agent:String,thought:String,tokens:u32},
    AgentAction{agent:String,kind:String,params:serde_json::Value},
    AgentTool{agent:String,tool:String,input:String,output:String},
    AgentSpawned{agent:String,kind:String}, AgentDestroyed{agent:String},
    CouncilDecision{decision:String,agents:Vec<String>,confidence:f32},
    AiApiCall{agent:String,provider:String,model:String,tokens:u32,cost:f64,latency_ms:f32},
    AiError{agent:String,provider:String,error:String},
    TokenBudgetLow{agent:String,remaining:u32},
    TokenBudgetExhausted{agent:String},
    Custom{kind:String,data:serde_json::Value},
}

type GameHandler  = Box<dyn Fn(&GameEvent)  + Send + Sync>;
type AgentHandler = Box<dyn Fn(&AgentEvent) + Send + Sync>;

pub struct EventBus {
    game_handlers:  Vec<(String,GameHandler)>,
    agent_handlers: Vec<(String,AgentHandler)>,
    game_queue:     std::sync::Mutex<Vec<GameEvent>>,
    agent_queue:    std::sync::Mutex<Vec<AgentEvent>>,
    pub total_game:  u64,
    pub total_agent: u64,
}

impl EventBus {
    pub fn new()->Self{Self{game_handlers:Vec::new(),agent_handlers:Vec::new(),game_queue:std::sync::Mutex::new(Vec::new()),agent_queue:std::sync::Mutex::new(Vec::new()),total_game:0,total_agent:0}}
    pub fn on_game<F:Fn(&GameEvent)+Send+Sync+'static>(&mut self,id:&str,f:F){self.game_handlers.push((id.to_string(),Box::new(f)));}
    pub fn on_agent<F:Fn(&AgentEvent)+Send+Sync+'static>(&mut self,id:&str,f:F){self.agent_handlers.push((id.to_string(),Box::new(f)));}
    pub fn emit_game(&mut self,ev:GameEvent){self.total_game+=1;for(_,h) in &self.game_handlers{h(&ev);}}
    pub fn emit_agent(&mut self,ev:AgentEvent){self.total_agent+=1;for(_,h) in &self.agent_handlers{h(&ev);}}
    pub fn queue_game(&self,ev:GameEvent){self.game_queue.lock().unwrap().push(ev);}
    pub fn queue_agent(&self,ev:AgentEvent){self.agent_queue.lock().unwrap().push(ev);}
    pub fn flush_game(&mut self){let evs:Vec<_>=self.game_queue.lock().unwrap().drain(..).collect();for ev in evs{self.emit_game(ev);}}
    pub fn flush_agent(&mut self){let evs:Vec<_>=self.agent_queue.lock().unwrap().drain(..).collect();for ev in evs{self.emit_agent(ev);}}
    pub fn off(&mut self,id:&str){self.game_handlers.retain(|(i,_)|i!=id);self.agent_handlers.retain(|(i,_)|i!=id);}
    pub fn total(&self)->u64{self.total_game+self.total_agent}
}

// ═══ PLUGIN TRAIT ════════════════════════════════════════════════
pub struct EngineContext {
    pub state:   EngineState,
    pub frame:   u64,
    pub delta:   f32,
    pub time:    f64,
    pub config:  HashMap<String,serde_json::Value>,
}

pub trait Plugin:Send+Sync {
    fn id(&self)          ->&str;
    fn name(&self)        ->&str;
    fn version(&self)     ->&str;
    fn description(&self) ->&str{""}
    fn on_load(&mut self, _ctx:&mut EngineContext){}
    fn on_enable(&mut self, _ctx:&mut EngineContext){}
    fn on_disable(&mut self, _ctx:&mut EngineContext){}
    fn on_unload(&mut self, _ctx:&mut EngineContext){}
    fn on_game_event(&mut self, _ev:&GameEvent, _ctx:&mut EngineContext){}
    fn on_update(&mut self, _delta:f32, _ctx:&mut EngineContext){}
    fn on_fixed_update(&mut self, _delta:f32, _ctx:&mut EngineContext){}
    fn provides_nodes(&self)->Vec<String>{Vec::new()}
    fn required_plugins(&self)->Vec<String>{Vec::new()}
}

pub struct PluginRegistry {
    pub plugins:   Vec<Box<dyn Plugin>>,
    pub enabled:   std::collections::HashSet<String>,
    pub load_order:Vec<String>,
}
impl PluginRegistry {
    pub fn new()->Self{Self{plugins:Vec::new(),enabled:std::collections::HashSet::new(),load_order:Vec::new()}}
    pub fn register(&mut self,mut p:Box<dyn Plugin>,ctx:&mut EngineContext){
        let id=p.id().to_string();
        tracing::info!("Plugin: {} v{}",p.name(),p.version());
        p.on_load(ctx);
        self.load_order.push(id.clone());
        self.enabled.insert(id.clone());
        self.plugins.push(p);
    }
    pub fn tick_all(&mut self,delta:f32,ctx:&mut EngineContext){
        for p in &mut self.plugins{if self.enabled.contains(p.id()){p.on_update(delta,ctx);}}
    }
    pub fn count(&self)->usize{self.plugins.len()}
    pub fn enabled_count(&self)->usize{self.enabled.len()}
}

// ═══ ENTITY DNA ══════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct EntityDocument {
    pub id:         String,
    pub kind:       EntityKind,
    pub name:       String,
    pub archetype:  String,
    pub tags:       Vec<String>,
    pub groups:     Vec<String>,
    pub owner:      Option<String>,
    pub spawned_at: DateTime<Utc>,
    pub scene_id:   String,
    pub persistent: bool,
    pub replicated: bool,
    pub active:     bool,
    pub generation: u32,
    pub meta:       HashMap<String,serde_json::Value>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum EntityKind {
    Player,Npc,Enemy,Boss,Creature,Item,Projectile,
    Trigger,Light,Camera,Effect,Prop,Terrain,
    Vehicle,Rope,Fluid,Destructible,Sound,Ui,Custom(String),
}
impl EntityDocument {
    pub fn new(id:&str,kind:EntityKind,name:&str,scene:&str)->Self{
        Self{id:id.to_string(),kind,name:name.to_string(),archetype:String::new(),tags:Vec::new(),groups:Vec::new(),owner:None,spawned_at:Utc::now(),scene_id:scene.to_string(),persistent:false,replicated:false,active:true,generation:0,meta:HashMap::new()}
    }
    pub fn has_tag(&self,t:&str)->bool{self.tags.iter().any(|s|s==t)}
    pub fn add_tag(&mut self,t:&str){if!self.has_tag(t){self.tags.push(t.to_string());}}
    pub fn remove_tag(&mut self,t:&str){self.tags.retain(|s|s!=t);}
    pub fn set_meta<V:serde::Serialize>(&mut self,k:&str,v:V){self.meta.insert(k.to_string(),serde_json::to_value(v).unwrap_or_default());}
    pub fn get_meta<V:serde::de::DeserializeOwned>(&self,k:&str)->Option<V>{self.meta.get(k).and_then(|v|serde_json::from_value(v.clone()).ok())}
}

// ═══ FRAME TIMER ═════════════════════════════════════════════════
#[derive(Debug,Clone)]
pub struct FrameTimer {
    pub frame:       u64,
    pub time:        f64,
    pub delta:       f32,
    pub fps:         f32,
    pub fps_history: std::collections::VecDeque<f32>,
    pub fixed_step:  f32,
    pub fixed_accum: f32,
    pub max_delta:   f32,
    pub time_scale:  f32,
    pub real_time:   f64,
    pub started_at:  std::time::Instant,
    pub frame_start: std::time::Instant,
}
impl FrameTimer {
    pub fn new(fps:f32)->Self{let n=std::time::Instant::now();Self{frame:0,time:0.0,delta:1.0/fps,fps,fps_history:std::collections::VecDeque::new(),fixed_step:1.0/60.0,fixed_accum:0.0,max_delta:0.1,time_scale:1.0,real_time:0.0,started_at:n,frame_start:n}}
    pub fn begin(&mut self){
        let now=std::time::Instant::now();
        let raw=now.duration_since(self.frame_start).as_secs_f32();
        self.frame_start=now;
        self.delta=(raw*self.time_scale).min(self.max_delta);
        self.real_time=now.duration_since(self.started_at).as_secs_f64();
        self.time+=self.delta as f64;
        self.fixed_accum+=self.delta;
        self.frame+=1;
        self.fps_history.push_back(1.0/raw.max(0.0001));
        if self.fps_history.len()>60{self.fps_history.pop_front();}
        self.fps=self.fps_history.iter().sum::<f32>()/self.fps_history.len() as f32;
    }
    pub fn fixed_steps(&mut self)->u32{
        let mut n=0;
        while self.fixed_accum>=self.fixed_step{self.fixed_accum-=self.fixed_step;n+=1;if n>8{self.fixed_accum=0.0;break;}}
        n
    }
    pub fn frame_ms(&self)->f32{self.delta*1000.0}
    pub fn uptime(&self)->f64{self.real_time}
    pub fn is_slow(&self)->bool{self.fps<50.0}
}

// ═══ GENESIS ENGINE ══════════════════════════════════════════════
pub struct GenesisEngine {
    pub state:    EngineState,
    pub timer:    FrameTimer,
    pub events:   EventBus,
    pub plugins:  PluginRegistry,
    pub entities: HashMap<String,EntityDocument>,
    pub config:   HashMap<String,serde_json::Value>,
    pub version:  &'static str,
}
impl GenesisEngine {
    pub fn new()->Self{Self{state:EngineState::Uninitialized,timer:FrameTimer::new(60.0),events:EventBus::new(),plugins:PluginRegistry::new(),entities:HashMap::new(),config:HashMap::new(),version:"0.1.0"}}
    pub fn boot(&mut self){
        self.state=EngineState::Booting{step:"Initialising".to_string(),progress:0.0};
        tracing::info!("GENESIS Engine v{} booting",self.version);
        self.events.emit_game(GameEvent::EngineReady);
        self.state=EngineState::Running{frame:0,fps:60.0};
        tracing::info!("Engine ready");
    }
    pub fn begin_frame(&mut self){
        self.timer.begin();
        self.events.flush_game();
        self.events.flush_agent();
        let f=self.timer.frame;let d=self.timer.delta;
        self.events.emit_game(GameEvent::FrameStart{frame:f,delta:d});
        if let EngineState::Running{frame,fps}=&mut self.state{*frame=f;*fps=self.timer.fps;}
    }
    pub fn end_frame(&mut self){
        let f=self.timer.frame;let d=self.timer.frame_ms();
        self.events.emit_game(GameEvent::FrameEnd{frame:f,dt_ms:d});
    }
    pub fn shutdown(&mut self){
        tracing::info!("Engine shutdown — {} frames, {:.1}s uptime",self.timer.frame,self.timer.uptime());
        self.events.emit_game(GameEvent::Shutdown);
        self.state=EngineState::ShuttingDown;
    }
    pub fn spawn(&mut self,doc:EntityDocument)->String{
        let id=doc.id.clone();let arch=doc.archetype.clone();
        self.entities.insert(id.clone(),doc);
        self.events.emit_game(GameEvent::EntitySpawned{id:id.clone(),arch});
        id
    }
    pub fn destroy(&mut self,id:&str){
        if self.entities.remove(id).is_some(){self.events.emit_game(GameEvent::EntityDestroyed{id:id.to_string()});}
    }
    pub fn get(&self,id:&str)->Option<&EntityDocument>{self.entities.get(id)}
    pub fn entity_count(&self)->usize{self.entities.len()}
    pub fn is_running(&self)->bool{self.state.is_running()}
    pub fn fps(&self)->f32{self.timer.fps}
    pub fn frame(&self)->u64{self.timer.frame}
    pub fn delta(&self)->f32{self.timer.delta}
    pub fn time(&self)->f64{self.timer.time}
}
impl Default for GenesisEngine{fn default()->Self{Self::new()}}
extern crate tracing;
