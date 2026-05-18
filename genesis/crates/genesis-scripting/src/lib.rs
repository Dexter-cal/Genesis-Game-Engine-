//! Scripting — ChronoScript language, Lua 5.4, Rhai, visual node graph, 80+ builtins
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ SCRIPT KINDS ════════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum ScriptLang { ChronoScript, Lua54, Rhai, VisualGraph, Wasm }

impl ScriptLang {
    pub fn extension(&self) -> &str {
        match self { Self::ChronoScript=>"cv", Self::Lua54=>"lua", Self::Rhai=>"rhai", Self::VisualGraph=>"vg", Self::Wasm=>"wasm" }
    }
    pub fn name(&self) -> &str {
        match self { Self::ChronoScript=>"ChronoScript", Self::Lua54=>"Lua 5.4", Self::Rhai=>"Rhai", Self::VisualGraph=>"Visual Graph", Self::Wasm=>"WebAssembly" }
    }
}

// ═══ SCRIPT SOURCE ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Script {
    pub id:       String,
    pub name:     String,
    pub path:     String,
    pub lang:     ScriptLang,
    pub source:   String,
    pub compiled: bool,
    pub errors:   Vec<ScriptError>,
    pub warnings: Vec<ScriptWarning>,
    pub metadata: ScriptMeta,
    pub version:  u32,
    pub modified: DateTime<Utc>,
    pub checksum: String,
    pub exported_fns: Vec<ExportedFn>,
    pub attached_to:  Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScriptError { pub line:u32, pub col:u32, pub msg:String, pub kind:ErrorKind }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScriptWarning { pub line:u32, pub col:u32, pub msg:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ErrorKind { Syntax, Type, Runtime, Reference, Permission, Timeout }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScriptMeta {
    pub description: String,
    pub author:      String,
    pub category:    ScriptCategory,
    pub tags:        Vec<String>,
    pub requires:    Vec<String>,
    pub events:      Vec<String>,  // which events this script handles
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ScriptCategory { Character, Npc, Item, Environment, Ui, System, Utility, Gameplay, Custom(String) }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ExportedFn { pub name:String, pub params:Vec<String>, pub return_type:String, pub doc:String }

// ═══ CHRONOSCRIPT BUILTINS (80+) ══════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Builtin {
    pub name:        String,
    pub category:    BuiltinCategory,
    pub params:      Vec<BuiltinParam>,
    pub return_type: String,
    pub description: String,
    pub example:     String,
    pub since:       String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BuiltinParam { pub name:String, pub type_:String, pub optional:bool, pub default_:Option<String> }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum BuiltinCategory {
    World, Physics, Audio, Input, Ui, Math, String_, IO,
    Time, Debug, Network, Ai, Scene, Animation, Camera,
    Particles, Random, Collections, Signal, Storage, Custom(String),
}

pub fn all_builtins() -> Vec<Builtin> {
    let b = |name:&str, cat:BuiltinCategory, params:Vec<(&str,&str)>, ret:&str, desc:&str, ex:&str| Builtin {
        name:name.to_string(), category:cat,
        params:params.iter().map(|(n,t)|BuiltinParam{name:n.to_string(),type_:t.to_string(),optional:false,default_:None}).collect(),
        return_type:ret.to_string(), description:desc.to_string(), example:ex.to_string(), since:"0.1.0".to_string(),
    };
    vec![
        // World
        b("world.spawn",     BuiltinCategory::World, vec![("prefab","str"),("position","vec3")], "Entity", "Spawn a prefab at position", "world.spawn(\"enemy/troll\", [10,0,5])"),
        b("world.get_node",  BuiltinCategory::World, vec![("path","str")],                       "Entity", "Get node by scene path",      "world.get_node(\"Player/Weapon\")"),
        b("world.find_tag",  BuiltinCategory::World, vec![("tag","str")],                        "List",   "Find all entities with tag",  "world.find_tag(\"enemy\")"),
        b("world.distance",  BuiltinCategory::World, vec![("a","Entity"),("b","Entity")],        "float",  "Distance between entities",   "world.distance(self, player)"),
        b("world.destroy",   BuiltinCategory::World, vec![("entity","Entity")],                  "void",   "Destroy entity",              "world.destroy(enemy)"),
        b("world.exists",    BuiltinCategory::World, vec![("entity","Entity")],                  "bool",   "Check entity exists",         "world.exists(target)"),
        b("world.set_parent",BuiltinCategory::World, vec![("child","Entity"),("parent","Entity")],"void",  "Set parent entity",           "world.set_parent(item, player)"),
        // Physics
        b("physics.apply_force",    BuiltinCategory::Physics, vec![("entity","Entity"),("force","vec3")],   "void","Apply force",      "physics.apply_force(self, [0,500,0])"),
        b("physics.apply_impulse",  BuiltinCategory::Physics, vec![("entity","Entity"),("impulse","vec3")], "void","Apply impulse",    "physics.apply_impulse(self, [0,10,0])"),
        b("physics.set_velocity",   BuiltinCategory::Physics, vec![("entity","Entity"),("vel","vec3")],     "void","Set velocity",     "physics.set_velocity(self, [0,0,5])"),
        b("physics.get_velocity",   BuiltinCategory::Physics, vec![("entity","Entity")],                    "vec3","Get velocity",     "var v = physics.get_velocity(self)"),
        b("physics.raycast",        BuiltinCategory::Physics, vec![("origin","vec3"),("dir","vec3"),("dist","float")],"RayHit","Cast ray","var hit = physics.raycast(pos, fwd, 100)"),
        b("physics.overlap_sphere", BuiltinCategory::Physics, vec![("center","vec3"),("radius","float")],   "List","Sphere overlap",   "var hits = physics.overlap_sphere(pos, 5.0)"),
        b("physics.gravity",        BuiltinCategory::Physics, vec![("g","vec3")],                           "void","Set world gravity","physics.gravity([0,-9.81,0])"),
        b("physics.is_grounded",    BuiltinCategory::Physics, vec![("entity","Entity")],                    "bool","Check if on floor","physics.is_grounded(self)"),
        // Audio
        b("audio.play",         BuiltinCategory::Audio, vec![("clip","str"),("vol","float")],         "SrcId","Play 2D sound",      "audio.play(\"coin.ogg\", 0.8)"),
        b("audio.play_at",      BuiltinCategory::Audio, vec![("clip","str"),("pos","vec3"),("vol","float")],"SrcId","Play spatial", "audio.play_at(\"footstep\",pos,1.0)"),
        b("audio.play_music",   BuiltinCategory::Audio, vec![("track","str"),("fade","float")],       "void", "Play music track",   "audio.play_music(\"boss_theme\", 2.0)"),
        b("audio.stop",         BuiltinCategory::Audio, vec![("src","SrcId")],                        "void", "Stop audio source",  "audio.stop(music_handle)"),
        b("audio.set_volume",   BuiltinCategory::Audio, vec![("bus","str"),("vol","float")],          "void", "Set bus volume",     "audio.set_volume(\"Music\", 0.5)"),
        b("audio.music_state",  BuiltinCategory::Audio, vec![("state","str")],                        "void", "Transition music",   "audio.music_state(\"combat\")"),
        // Input
        b("input.key",            BuiltinCategory::Input, vec![("key","str")],   "bool","Check key held",        "if input.key(\"space\"):"),
        b("input.just_pressed",   BuiltinCategory::Input, vec![("key","str")],   "bool","Key just pressed",      "if input.just_pressed(\"fire\"):"),
        b("input.just_released",  BuiltinCategory::Input, vec![("key","str")],   "bool","Key just released",     "if input.just_released(\"aim\"):"),
        b("input.action",         BuiltinCategory::Input, vec![("name","str")],  "bool","Check named action",    "if input.action(\"jump\"):"),
        b("input.move_dir",       BuiltinCategory::Input, vec![],                "vec2","Get WASD direction",    "var dir = input.move_dir()"),
        b("input.mouse_pos",      BuiltinCategory::Input, vec![],                "vec2","Get mouse position",    "var mp = input.mouse_pos()"),
        b("input.mouse_delta",    BuiltinCategory::Input, vec![],                "vec2","Get mouse movement",    "var md = input.mouse_delta()"),
        b("input.gamepad_axis",   BuiltinCategory::Input, vec![("pad","int"),("axis","str")],"float","Gamepad axis","input.gamepad_axis(0,\"LX\")"),
        b("input.vibrate",        BuiltinCategory::Input, vec![("pad","int"),("low","float"),("high","float"),("secs","float")],"void","Rumble gamepad","input.vibrate(0,0.5,0.5,0.3)"),
        // UI
        b("ui.show",          BuiltinCategory::Ui, vec![("screen","str")],              "void","Show UI screen",       "ui.show(\"inventory\")"),
        b("ui.hide",          BuiltinCategory::Ui, vec![("screen","str")],              "void","Hide UI screen",       "ui.hide(\"hud\")"),
        b("ui.notify",        BuiltinCategory::Ui, vec![("text","str"),("dur","float")], "void","Show notification",   "ui.notify(\"Quest complete!\", 3.0)"),
        b("ui.dialogue",      BuiltinCategory::Ui, vec![("speaker","str"),("text","str")],"void","Show dialogue",      "ui.dialogue(\"Elder\", \"Welcome.\")"),
        b("ui.damage_number", BuiltinCategory::Ui, vec![("val","float"),("pos","vec3"),("crit","bool")],"void","Damage number","ui.damage_number(42.0,pos,true)"),
        b("ui.set_health",    BuiltinCategory::Ui, vec![("pct","float")],               "void","Update health bar",    "ui.set_health(player.hp/player.max_hp)"),
        b("ui.waypoint",      BuiltinCategory::Ui, vec![("id","str"),("pos","vec3"),("label","str")],"void","Add waypoint","ui.waypoint(\"quest\",target_pos,\"Talk to Guard\")"),
        // Math
        b("math.lerp",       BuiltinCategory::Math, vec![("a","float"),("b","float"),("t","float")],"float","Linear interpolate","math.lerp(0,100,0.5)"),
        b("math.clamp",      BuiltinCategory::Math, vec![("v","float"),("lo","float"),("hi","float")],"float","Clamp value",      "math.clamp(hp,0,100)"),
        b("math.normalize",  BuiltinCategory::Math, vec![("v","vec3")],                              "vec3", "Normalize vector",  "math.normalize(dir)"),
        b("math.dot",        BuiltinCategory::Math, vec![("a","vec3"),("b","vec3")],                 "float","Dot product",       "math.dot(fwd, to_enemy)"),
        b("math.cross",      BuiltinCategory::Math, vec![("a","vec3"),("b","vec3")],                 "vec3", "Cross product",     "math.cross(up, fwd)"),
        b("math.sin",        BuiltinCategory::Math, vec![("x","float")],                             "float","Sine",              "math.sin(time * 2.0)"),
        b("math.cos",        BuiltinCategory::Math, vec![("x","float")],                             "float","Cosine",            "math.cos(angle)"),
        b("math.abs",        BuiltinCategory::Math, vec![("x","float")],                             "float","Absolute value",    "math.abs(-5.0)"),
        b("math.pow",        BuiltinCategory::Math, vec![("x","float"),("n","float")],               "float","Power",             "math.pow(2.0, 10.0)"),
        b("math.sqrt",       BuiltinCategory::Math, vec![("x","float")],                             "float","Square root",       "math.sqrt(9.0)"),
        b("math.floor",      BuiltinCategory::Math, vec![("x","float")],                             "float","Floor",             "math.floor(3.7)"),
        b("math.ceil",       BuiltinCategory::Math, vec![("x","float")],                             "float","Ceiling",           "math.ceil(3.2)"),
        b("math.round",      BuiltinCategory::Math, vec![("x","float")],                             "float","Round",             "math.round(3.5)"),
        // Random
        b("rng.float",       BuiltinCategory::Random, vec![("lo","float"),("hi","float")],        "float","Random float",     "rng.float(0.0, 1.0)"),
        b("rng.int",         BuiltinCategory::Random, vec![("lo","int"),("hi","int")],             "int",  "Random integer",   "rng.int(1, 6)"),
        b("rng.pick",        BuiltinCategory::Random, vec![("list","List")],                       "any",  "Pick from list",   "rng.pick([\"a\",\"b\",\"c\"])"),
        b("rng.chance",      BuiltinCategory::Random, vec![("pct","float")],                       "bool", "Chance 0-1",       "if rng.chance(0.25):"),
        b("rng.seed",        BuiltinCategory::Random, vec![("s","int")],                           "void", "Set RNG seed",     "rng.seed(42)"),
        // Time
        b("time.now",        BuiltinCategory::Time, vec![], "float","Current time in seconds","var t = time.now()"),
        b("time.delta",      BuiltinCategory::Time, vec![], "float","Frame delta time",        "self.move(speed * time.delta())"),
        b("time.frame",      BuiltinCategory::Time, vec![], "int",  "Current frame number",   "if time.frame() % 60 == 0:"),
        b("time.hour",       BuiltinCategory::Time, vec![], "float","In-game hour 0-24",      "if time.hour() > 20.0:"),
        b("time.is_day",     BuiltinCategory::Time, vec![], "bool", "Is it daytime",          "if time.is_day():"),
        // Debug
        b("debug.log",       BuiltinCategory::Debug, vec![("msg","str")],                    "void","Log message",   "debug.log(\"HP: \" + hp)"),
        b("debug.warn",      BuiltinCategory::Debug, vec![("msg","str")],                    "void","Log warning",   "debug.warn(\"Low health!\")"),
        b("debug.draw_line", BuiltinCategory::Debug, vec![("a","vec3"),("b","vec3"),("col","color"),("dur","float")],"void","Draw debug line","debug.draw_line(pos,target,[1,0,0,1],2.0)"),
        b("debug.draw_sphere",BuiltinCategory::Debug,vec![("c","vec3"),("r","float"),("col","color")],"void","Draw debug sphere","debug.draw_sphere(pos,1.0,[0,1,0,1])"),
        b("debug.label",     BuiltinCategory::Debug, vec![("pos","vec3"),("text","str")],    "void","World space label","debug.label(self.pos,\"HP: \"+hp)"),
        // AI
        b("ai.chat",         BuiltinCategory::Ai, vec![("system","str"),("user","str")],    "str", "Call LLM",          "var r = ai.chat(system:npc_prompt, user:player_msg)"),
        b("ai.npc",          BuiltinCategory::Ai, vec![("desc","str")],                     "NPC", "Generate NPC",      "var npc = ai.npc(\"wise old wizard\")"),
        b("ai.quest",        BuiltinCategory::Ai, vec![("desc","str")],                     "Quest","Generate quest",   "var q = ai.quest(\"find missing sword\")"),
        b("ai.item",         BuiltinCategory::Ai, vec![("desc","str")],                     "Item", "Generate item",    "var it = ai.item(\"cursed dagger\")"),
        b("ai.voice",        BuiltinCategory::Ai, vec![("text","str"),("voice_id","str")],  "Audio","Generate TTS",     "ai.voice(line, npc.voice_id)"),
        b("ai.classify",     BuiltinCategory::Ai, vec![("text","str"),("labels","List")],   "str",  "Classify text",    "ai.classify(msg,[\"friendly\",\"hostile\"])"),
        // Camera
        b("camera.shake",     BuiltinCategory::Camera, vec![("intensity","float"),("dur","float")],"void","Camera shake","camera.shake(0.5, 0.3)"),
        b("camera.look_at",   BuiltinCategory::Camera, vec![("target","vec3"),("dur","float")],   "void","Look at point","camera.look_at(boss.pos, 0.5)"),
        b("camera.set_fov",   BuiltinCategory::Camera, vec![("fov","float"),("dur","float")],     "void","Change FOV",   "camera.set_fov(90.0, 0.2)"),
        b("camera.offset",    BuiltinCategory::Camera, vec![("offset","vec3")],                    "void","Camera offset","camera.offset([0,2,-5])"),
        // Signal
        b("signal.emit",      BuiltinCategory::Signal, vec![("event","str"),("data","dict")],  "void","Emit signal",     "signal.emit(\"enemy_killed\",{id:self.id})"),
        b("signal.connect",   BuiltinCategory::Signal, vec![("event","str"),("fn","Callable")],"void","Connect signal",  "signal.connect(\"player_died\", on_death)"),
        b("signal.disconnect",BuiltinCategory::Signal, vec![("event","str"),("fn","Callable")],"void","Remove listener", "signal.disconnect(\"hit\", handler)"),
        // Storage
        b("storage.save",    BuiltinCategory::Storage, vec![("key","str"),("val","any")],  "void","Save persistent data",  "storage.save(\"highscore\", score)"),
        b("storage.load",    BuiltinCategory::Storage, vec![("key","str"),("default","any")],"any","Load persistent data", "var hs = storage.load(\"highscore\",0)"),
        b("storage.delete",  BuiltinCategory::Storage, vec![("key","str")],                "void","Delete stored key",    "storage.delete(\"temp_data\")"),
        b("storage.has",     BuiltinCategory::Storage, vec![("key","str")],                "bool","Check if key exists",  "if storage.has(\"completed_tutorial\"):"),
        // Animation
        b("anim.play",       BuiltinCategory::Animation, vec![("name","str"),("blend","float")],    "void","Play animation",     "anim.play(\"run\", 0.2)"),
        b("anim.stop",       BuiltinCategory::Animation, vec![("name","str"),("blend","float")],    "void","Stop animation",     "anim.stop(\"idle\", 0.1)"),
        b("anim.set_param",  BuiltinCategory::Animation, vec![("name","str"),("val","float")],      "void","Set blend param",    "anim.set_param(\"speed\", move_speed)"),
        b("anim.is_playing", BuiltinCategory::Animation, vec![("name","str")],                      "bool","Check if playing",   "if anim.is_playing(\"attack\"):"),
        // Particles
        b("fx.spawn",        BuiltinCategory::Particles, vec![("effect","str"),("pos","vec3")],     "void","Spawn particle effect","fx.spawn(\"explosion\",pos)"),
        b("fx.attach",       BuiltinCategory::Particles, vec![("effect","str"),("entity","Entity")], "void","Attach particles",   "fx.attach(\"fire\", torch)"),
        b("fx.stop",         BuiltinCategory::Particles, vec![("effect","str"),("entity","Entity")], "void","Stop particles",     "fx.stop(\"fire\", torch)"),
        // Scene
        b("scene.load",      BuiltinCategory::Scene, vec![("id","str")],          "void","Load scene",        "scene.load(\"dungeon_01\")"),
        b("scene.reload",    BuiltinCategory::Scene, vec![],                       "void","Reload scene",      "scene.reload()"),
        b("scene.back",      BuiltinCategory::Scene, vec![],                       "void","Previous scene",    "scene.back()"),
        b("scene.preload",   BuiltinCategory::Scene, vec![("id","str")],           "void","Preload scene",     "scene.preload(\"boss_arena\")"),
    ]
}

// ═══ VISUAL NODE GRAPH ════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct VisualGraph {
    pub id:     String,
    pub name:   String,
    pub nodes:  Vec<GraphNode>,
    pub edges:  Vec<GraphEdge>,
    pub vars:   HashMap<String,GraphVar>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GraphNode {
    pub id:       String,
    pub kind:     GraphNodeKind,
    pub pos:      [f32;2],
    pub title:    String,
    pub comment:  String,
    pub inputs:   Vec<NodePort>,
    pub outputs:  Vec<NodePort>,
    pub params:   HashMap<String,serde_json::Value>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum GraphNodeKind {
    Event{name:String}, Condition, Branch, Loop,
    GetVar{name:String}, SetVar{name:String},
    CallBuiltin{name:String}, CallScript{path:String,fn_:String},
    Math{op:MathOp}, Logic{op:LogicOp}, Compare{op:CmpOp},
    Sequence, Selector, Parallel,
    Print, Comment{text:String},
    Custom{type_id:String},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum MathOp { Add, Sub, Mul, Div, Mod, Pow, Min, Max }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum LogicOp { And, Or, Not, Xor }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CmpOp { Eq, Ne, Lt, Le, Gt, Ge }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NodePort { pub id:String, pub name:String, pub type_:String, pub optional:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GraphEdge { pub from_node:String, pub from_port:String, pub to_node:String, pub to_port:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GraphVar { pub name:String, pub type_:String, pub value:serde_json::Value, pub is_public:bool }

// ═══ SCRIPT RUNTIME ══════════════════════════════════════════════
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum RuntimeState { Idle, Running, Paused, Error(String), Finished }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ScriptInstance {
    pub script_id:   String,
    pub entity_id:   String,
    pub state:       RuntimeState,
    pub globals:     HashMap<String,serde_json::Value>,
    pub call_stack:  Vec<String>,
    pub exec_time_ms:f32,
    pub call_count:  u64,
    pub error_count: u32,
    pub yielded:     bool,
    pub yield_until: Option<f32>,
}

pub struct ScriptRuntime {
    pub scripts:     HashMap<String,Script>,
    pub instances:   HashMap<String,ScriptInstance>,
    pub builtins:    Vec<Builtin>,
    pub graphs:      HashMap<String,VisualGraph>,
    pub budget_ms:   f32,
    pub hot_reload:  bool,
    pub sandbox:     bool,
    pub total_calls: u64,
    pub total_errors:u32,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        Self {
            scripts:HashMap::new(), instances:HashMap::new(),
            builtins:all_builtins(), graphs:HashMap::new(),
            budget_ms:2.0, hot_reload:true, sandbox:true,
            total_calls:0, total_errors:0,
        }
    }

    pub fn register(&mut self, script:Script) {
        tracing::info!("Registered script: {} ({:?})", script.name, script.lang);
        self.scripts.insert(script.id.clone(), script);
    }

    pub fn instantiate(&mut self, script_id:&str, entity_id:&str) -> String {
        let inst_id = format!("inst_{}_{}", script_id, entity_id);
        let inst = ScriptInstance {
            script_id:script_id.to_string(), entity_id:entity_id.to_string(),
            state:RuntimeState::Idle, globals:HashMap::new(), call_stack:Vec::new(),
            exec_time_ms:0.0, call_count:0, error_count:0, yielded:false, yield_until:None,
        };
        self.instances.insert(inst_id.clone(), inst);
        inst_id
    }

    pub fn call(&mut self, inst_id:&str, fn_name:&str, _args:&[serde_json::Value]) -> Result<serde_json::Value, String> {
        if let Some(inst) = self.instances.get_mut(inst_id) {
            if matches!(inst.state, RuntimeState::Error(_)) { return Err("Script in error state".to_string()); }
            inst.state = RuntimeState::Running;
            inst.call_stack.push(fn_name.to_string());
            inst.call_count += 1;
            self.total_calls += 1;
            // Real impl: dispatch to Lua/Rhai/ChronoScript interpreter
            if let Some(inst) = self.instances.get_mut(inst_id) {
                inst.call_stack.pop();
                inst.state = RuntimeState::Idle;
            }
            Ok(serde_json::Value::Null)
        } else { Err(format!("Instance not found: {}", inst_id)) }
    }

    pub fn emit_event(&mut self, event:&str, data:serde_json::Value) {
        tracing::debug!("Script event: {}", event);
        let inst_ids:Vec<_>=self.instances.keys().cloned().collect();
        for id in inst_ids {
            let _ = self.call(&id, &format!("on_{}", event), &[data.clone()]);
        }
    }

    pub fn tick(&mut self, delta:f32) {
        let now_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f32();
        let inst_ids:Vec<_>=self.instances.keys().cloned().collect();
        for id in inst_ids {
            let should_run = self.instances.get(&id).map(|i| {
                match i.yield_until { Some(t)=>now_secs>=t, None=>true }
                && !matches!(i.state, RuntimeState::Error(_))
            }).unwrap_or(false);
            if should_run {
                if let Some(inst) = self.instances.get_mut(&id) { inst.yielded=false; inst.yield_until=None; }
                let _ = self.call(&id, "tick", &[serde_json::json!(delta)]);
            }
        }
    }

    pub fn builtin_count(&self) -> usize { self.builtins.len() }
    pub fn script_count(&self) -> usize { self.scripts.len() }
    pub fn instance_count(&self) -> usize { self.instances.len() }
    pub fn find_builtin(&self, name:&str) -> Option<&Builtin> { self.builtins.iter().find(|b|b.name==name) }
}

impl Default for ScriptRuntime { fn default() -> Self { Self::new() } }
extern crate tracing;
