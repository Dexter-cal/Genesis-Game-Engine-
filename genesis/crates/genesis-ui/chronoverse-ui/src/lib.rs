//! UI System — immediate mode GUI, HUD, menus, waypoints, damage numbers, dialogue
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ── Widget System ─────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Widget {
    Panel    { id:String, rect:Rect, style:PanelStyle, children:Vec<Widget>, visible:bool },
    Text     { id:String, content:String, pos:Pos2, style:TextStyle },
    Button   { id:String, label:String, rect:Rect, style:ButtonStyle, hovered:bool, pressed:bool },
    Image    { id:String, texture:String, rect:Rect, tint:[f32;4] },
    ProgressBar{ id:String, rect:Rect, value:f32, max:f32, color:[f32;4], bg:[f32;4] },
    Slider   { id:String, rect:Rect, value:f32, min:f32, max:f32, vertical:bool },
    Checkbox { id:String, label:String, pos:Pos2, checked:bool },
    Input    { id:String, rect:Rect, text:String, placeholder:String, focused:bool },
    Dropdown { id:String, rect:Rect, options:Vec<String>, selected:usize, open:bool },
    Tooltip  { text:String, pos:Pos2, visible:bool },
    Spacer   { height:f32 },
}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct Rect { pub x:f32, pub y:f32, pub w:f32, pub h:f32 }
impl Rect {
    pub fn new(x:f32,y:f32,w:f32,h:f32)->Self{Self{x,y,w,h}}
    pub fn contains(&self,p:Pos2)->bool{ p.x>=self.x&&p.x<=self.x+self.w&&p.y>=self.y&&p.y<=self.y+self.h }
}
#[derive(Debug,Clone,Copy,Serialize,Deserialize,Default)]
pub struct Pos2 { pub x:f32, pub y:f32 }
impl Pos2 { pub fn new(x:f32,y:f32)->Self{Self{x,y}} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PanelStyle { pub bg:[f32;4], pub border:[f32;4], pub border_w:f32, pub radius:f32, pub shadow:f32, pub blur:bool }
impl Default for PanelStyle { fn default()->Self{Self{bg:[0.08,0.09,0.12,0.92],border:[1.0,1.0,1.0,0.08],border_w:1.0,radius:8.0,shadow:16.0,blur:true}} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TextStyle { pub color:[f32;4], pub size:f32, pub font:String, pub bold:bool, pub italic:bool, pub shadow:bool }
impl Default for TextStyle { fn default()->Self{Self{color:[1.0,1.0,1.0,1.0],size:14.0,font:"DM Sans".to_string(),bold:false,italic:false,shadow:false}} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ButtonStyle { pub bg:[f32;4], pub hover:[f32;4], pub pressed:[f32;4], pub text:[f32;4], pub radius:f32 }
impl Default for ButtonStyle { fn default()->Self{Self{bg:[0.3,0.6,1.0,0.15],hover:[0.3,0.6,1.0,0.25],pressed:[0.3,0.6,1.0,0.4],text:[1.0,1.0,1.0,1.0],radius:6.0}} }

// ── Screen / Canvas ───────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Screen {
    pub id:       String,
    pub name:     String,
    pub kind:     ScreenKind,
    pub widgets:  Vec<Widget>,
    pub visible:  bool,
    pub modal:    bool,
    pub pause_game:bool,
    pub z_order:  u32,
    pub anim_in:  Option<ScreenAnim>,
    pub anim_out: Option<ScreenAnim>,
    pub scripts:  Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ScreenKind {
    MainMenu, PauseMenu, Hud, Inventory, Dialogue, LoadingScreen,
    Credits, Settings, CharacterSelect, GameOver, Victory,
    Shop, Cutscene, Tutorial, Custom(String),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum ScreenAnim { FadeIn{dur:f32}, SlideIn{dir:String,dur:f32}, ScaleIn{dur:f32}, Custom(String) }

// ── HUD Elements ──────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct HudConfig {
    pub health_bar:    BarConfig,
    pub stamina_bar:   BarConfig,
    pub mana_bar:      BarConfig,
    pub xp_bar:        BarConfig,
    pub minimap:       MinimapConfig,
    pub crosshair:     CrosshairConfig,
    pub damage_numbers:DamageNumberConfig,
    pub waypoints:     WaypointConfig,
    pub notifications: NotificationConfig,
    pub compass:       bool,
    pub hotbar:        HotbarConfig,
    pub objectives:    bool,
    pub boss_bar:      bool,
    pub buff_icons:    bool,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BarConfig { pub enabled:bool, pub x:f32, pub y:f32, pub w:f32, pub h:f32, pub color:[f32;4], pub bg:[f32;4], pub smooth:bool, pub show_text:bool }
impl BarConfig {
    pub fn health()  ->Self{Self{enabled:true,x:20.0,y:-60.0,w:200.0,h:12.0,color:[0.9,0.2,0.2,1.0],bg:[0.2,0.05,0.05,0.8],smooth:true,show_text:false}}
    pub fn stamina() ->Self{Self{enabled:true,x:20.0,y:-40.0,w:160.0,h:8.0, color:[0.2,0.8,0.4,1.0],bg:[0.05,0.2,0.1,0.8],smooth:true,show_text:false}}
    pub fn mana()    ->Self{Self{enabled:true,x:20.0,y:-50.0,w:160.0,h:8.0, color:[0.3,0.5,0.9,1.0],bg:[0.05,0.1,0.25,0.8],smooth:true,show_text:false}}
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct MinimapConfig { pub enabled:bool, pub size:f32, pub zoom:f32, pub rotation:bool, pub show_enemies:bool, pub show_items:bool, pub show_npcs:bool }
impl Default for MinimapConfig { fn default()->Self{Self{enabled:true,size:180.0,zoom:1.0,rotation:true,show_enemies:true,show_items:false,show_npcs:true}} }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CrosshairConfig { pub enabled:bool, pub kind:CrosshairKind, pub color:[f32;4], pub size:f32, pub dynamic:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum CrosshairKind { Cross, Dot, Circle, T, Custom(String) }
impl Default for CrosshairConfig { fn default()->Self{Self{enabled:true,kind:CrosshairKind::Cross,color:[1.0,1.0,1.0,0.8],size:12.0,dynamic:true}} }

// ── Damage Numbers ────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DamageNumberConfig { pub enabled:bool, pub crit_scale:f32, pub crit_color:[f32;4], pub normal_color:[f32;4], pub float_speed:f32, pub lifetime:f32, pub font_size:f32 }
impl Default for DamageNumberConfig { fn default()->Self{Self{enabled:true,crit_scale:1.5,crit_color:[1.0,0.8,0.0,1.0],normal_color:[1.0,1.0,1.0,1.0],float_speed:40.0,lifetime:1.5,font_size:20.0}} }

#[derive(Debug,Clone)]
pub struct DamageNumber { pub value:f32, pub crit:bool, pub pos:[f32;3], pub vel:[f32;2], pub alpha:f32, pub age:f32, pub kind:DmgKind }
#[derive(Debug,Clone)]
pub enum DmgKind { Physical, Fire, Ice, Lightning, Poison, Heal, Shield, Miss }

// ── Waypoints ─────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct WaypointConfig { pub enabled:bool, pub max_distance:f32, pub size:f32, pub show_distance:bool }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Waypoint { pub id:String, pub world_pos:[f32;3], pub label:String, pub icon:String, pub color:[f32;4], pub visible:bool, pub pulse:bool, pub distance:f32 }

// ── Notifications ─────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct NotificationConfig { pub enabled:bool, pub position:String, pub duration:f32, pub max:u32, pub stack:bool }
#[derive(Debug,Clone)]
pub struct Notification { pub id:String, pub text:String, pub icon:Option<String>, pub color:[f32;4], pub age:f32, pub duration:f32 }

// ── Hotbar ────────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct HotbarConfig { pub enabled:bool, pub slots:u32, pub size:f32, pub show_keybind:bool }
impl Default for HotbarConfig { fn default()->Self{Self{enabled:true,slots:8,size:56.0,show_keybind:true}} }

// ── Dialogue ──────────────────────────────────────────────────────
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueNode {
    pub id:          String,
    pub speaker:     String,
    pub portrait:    Option<String>,
    pub text:        String,
    pub audio:       Option<String>,
    pub choices:     Vec<DialogueChoice>,
    pub auto_advance:bool,
    pub advance_secs:f32,
    pub camera_hint: Option<String>,
    pub trigger_on_enter:Option<String>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DialogueChoice { pub id:String, pub text:String, pub next:Option<String>, pub condition:Option<String>, pub consequence:Option<String> }

// ── UI Manager ────────────────────────────────────────────────────
pub struct UiManager {
    pub screens:      HashMap<String,Screen>,
    pub active_stack: Vec<String>,           // stack for modal screens
    pub hud:          HudConfig,
    pub damage_nums:  Vec<DamageNumber>,
    pub waypoints:    Vec<Waypoint>,
    pub notifications:Vec<Notification>,
    pub dialogue:     Option<DialogueState>,
    pub loading:      Option<LoadingState>,
    pub theme:        UiTheme,
    pub locale:       String,
    pub total_opens:  u64,
    pub screen_w:     u32,
    pub screen_h:     u32,
}

#[derive(Debug,Clone)]
pub struct DialogueState { pub current_node:String, pub speaker:String, pub visible_text:String, pub full_text:String, pub typing_t:f32, pub typing_speed:f32, pub choices:Vec<DialogueChoice> }
#[derive(Debug,Clone)]
pub struct LoadingState { pub progress:f32, pub tip:String, pub bg_image:Option<String> }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct UiTheme { pub primary:[f32;4], pub secondary:[f32;4], pub bg:[f32;4], pub surface:[f32;4], pub text:[f32;4], pub accent:[f32;4] }
impl Default for UiTheme { fn default()->Self{Self{primary:[0.3,0.6,1.0,1.0],secondary:[0.6,0.4,1.0,1.0],bg:[0.04,0.05,0.08,1.0],surface:[0.08,0.09,0.13,1.0],text:[0.9,0.93,1.0,1.0],accent:[0.0,0.9,0.78,1.0]}} }

impl UiManager {
    pub fn new(w:u32,h:u32)->Self{
        Self{
            screens:HashMap::new(), active_stack:Vec::new(),
            hud:HudConfig{
                health_bar:BarConfig::health(),stamina_bar:BarConfig::stamina(),
                mana_bar:BarConfig::mana(),xp_bar:BarConfig::stamina(),
                minimap:MinimapConfig::default(),crosshair:CrosshairConfig::default(),
                damage_numbers:DamageNumberConfig::default(),
                waypoints:WaypointConfig{enabled:true,max_distance:500.0,size:28.0,show_distance:true},
                notifications:NotificationConfig{enabled:true,position:"top-right".to_string(),duration:4.0,max:5,stack:true},
                compass:true,hotbar:HotbarConfig::default(),objectives:true,boss_bar:false,buff_icons:true,
            },
            damage_nums:Vec::new(),waypoints:Vec::new(),notifications:Vec::new(),
            dialogue:None,loading:None,theme:UiTheme::default(),
            locale:"en".to_string(),total_opens:0,screen_w:w,screen_h:h,
        }
    }

    pub fn open(&mut self,id:&str)->bool{
        if !self.screens.contains_key(id){return false;}
        if !self.active_stack.contains(&id.to_string()){
            self.active_stack.push(id.to_string());
            self.total_opens+=1;
            tracing::debug!("Screen opened: {}",id);
        }
        true
    }
    pub fn close(&mut self,id:&str){ self.active_stack.retain(|s|s!=id); }
    pub fn close_top(&mut self){ self.active_stack.pop(); }
    pub fn is_open(&self,id:&str)->bool{ self.active_stack.contains(&id.to_string()) }
    pub fn top(&self)->Option<&str>{ self.active_stack.last().map(|s|s.as_str()) }
    pub fn is_modal_open(&self)->bool{
        self.active_stack.last().and_then(|id|self.screens.get(id)).map(|s|s.modal).unwrap_or(false)
    }
    pub fn push_damage(&mut self,v:f32,crit:bool,pos:[f32;3],kind:DmgKind){
        self.damage_nums.push(DamageNumber{value:v,crit,pos,vel:[rand_f()-0.5,1.0+rand_f()],alpha:1.0,age:0.0,kind});
    }
    pub fn notify(&mut self,text:&str,icon:Option<&str>,color:[f32;4]){
        while self.notifications.len()>=5{self.notifications.remove(0);}
        self.notifications.push(Notification{id:format!("n_{}",self.total_opens),text:text.to_string(),icon:icon.map(|s|s.to_string()),color,age:0.0,duration:4.0});
    }
    pub fn tick(&mut self,delta:f32){
        // Update damage numbers
        for d in &mut self.damage_nums { d.age+=delta; d.alpha=(1.0-d.age/d.duration()-0.3).max(0.0); d.pos[1]+=d.vel[1]*delta*30.0; }
        self.damage_nums.retain(|d|d.age<d.duration());
        // Update notifications
        for n in &mut self.notifications { n.age+=delta; }
        self.notifications.retain(|n|n.age<n.duration);
        // Typing effect for dialogue
        if let Some(dlg)=&mut self.dialogue {
            if dlg.visible_text.len()<dlg.full_text.len(){
                dlg.typing_t+=delta*dlg.typing_speed;
                let chars=dlg.typing_t as usize;
                dlg.visible_text=dlg.full_text.chars().take(chars).collect();
            }
        }
    }
}

trait HasDuration { fn duration(&self)->f32; }
impl HasDuration for DamageNumber { fn duration(&self)->f32{1.5} }

fn rand_f()->f32{
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos() as f32/u32::MAX as f32)*2.0-1.0
}
extern crate tracing;
