//! Input System — keyboard, mouse, gamepad, touch, VR, phone controller, haptics
use serde::{Serialize,Deserialize};
use std::collections::{HashMap,HashSet};
use chrono::{DateTime,Utc};

// ═══ KEY CODES ════════════════════════════════════════════════════
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum Key {
    // Letters
    A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z,
    // Numbers
    Num0,Num1,Num2,Num3,Num4,Num5,Num6,Num7,Num8,Num9,
    // Function
    F1,F2,F3,F4,F5,F6,F7,F8,F9,F10,F11,F12,
    // Navigation
    Up,Down,Left,Right,Home,End,PageUp,PageDown,
    // Special
    Space,Enter,Escape,Tab,Backspace,Delete,Insert,
    LShift,RShift,LCtrl,RCtrl,LAlt,RAlt,LSuper,RSuper,
    // Numpad
    Num0p,Num1p,Num2p,Num3p,Num4p,Num5p,Num6p,Num7p,Num8p,Num9p,
    NumAdd,NumSub,NumMul,NumDiv,NumEnter,NumDecimal,NumLock,
    // Punctuation
    Comma,Period,Slash,Backslash,Semicolon,Quote,
    LBracket,RBracket,Backtick,Minus,Equals,
    // Media
    MediaPlay,MediaStop,MediaNext,MediaPrev,VolumeUp,VolumeDown,VolumeMute,
    PrintScreen,ScrollLock,Pause,CapsLock,
    Unknown(u32),
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum MouseButton { Left, Right, Middle, X1, X2 }

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum GamepadButton {
    South, North, East, West,
    LBumper, RBumper, LTrigger, RTrigger,
    Select, Start, Home,
    LThumb, RThumb,
    DpadUp, DpadDown, DpadLeft, DpadRight,
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,Serialize,Deserialize)]
pub enum GamepadAxis { LX, LY, RX, RY, LTrigger, RTrigger }

// ═══ ACTION MAPPING ═══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct InputAction {
    pub name:     String,
    pub bindings: Vec<InputBinding>,
    pub held:     bool,
    pub just_pressed: bool,
    pub just_released:bool,
    pub strength: f32,     // 0-1 analog value
    pub deadzone: f32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum InputBinding {
    Key(Key), Mouse(MouseButton), GamepadBtn{pad:u32,btn:GamepadButton},
    GamepadAxis{pad:u32,axis:GamepadAxis,positive:bool,threshold:f32},
    Touch{fingers:u32,gesture:TouchGesture},
    Combination(Vec<InputBinding>),
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum TouchGesture { Tap, DoubleTap, LongPress, Swipe{dir:SwipeDir}, Pinch, Rotate }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SwipeDir { Up, Down, Left, Right }

impl InputAction {
    pub fn new(name:&str) -> Self {
        Self { name:name.to_string(), bindings:Vec::new(), held:false,
               just_pressed:false, just_released:false, strength:0.0, deadzone:0.1 }
    }
    pub fn with_key(mut self, k:Key) -> Self { self.bindings.push(InputBinding::Key(k)); self }
    pub fn with_mouse(mut self, b:MouseButton) -> Self { self.bindings.push(InputBinding::Mouse(b)); self }
    pub fn with_gamepad(mut self, pad:u32, b:GamepadButton) -> Self { self.bindings.push(InputBinding::GamepadBtn{pad,btn:b}); self }
}

// ═══ MOUSE STATE ══════════════════════════════════════════════════
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct MouseState {
    pub position:      [f32;2],
    pub delta:         [f32;2],
    pub scroll:        [f32;2],
    pub captured:      bool,
    pub visible:       bool,
    pub sensitivity:   f32,
    pub raw_input:     bool,
    pub buttons_held:  HashSet<u8>,
    pub buttons_just_pressed:  HashSet<u8>,
    pub buttons_just_released: HashSet<u8>,
}
impl MouseState {
    pub fn new() -> Self { Self { sensitivity:1.0, visible:true, ..Default::default() } }
    pub fn button_held(&self,b:MouseButton) -> bool { self.buttons_held.contains(&(b as u8)) }
    pub fn just_pressed(&self,b:MouseButton) -> bool { self.buttons_just_pressed.contains(&(b as u8)) }
}

// ═══ GAMEPAD STATE ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct GamepadState {
    pub id:           u32,
    pub name:         String,
    pub connected:    bool,
    pub buttons_held: HashSet<u8>,
    pub buttons_just_pressed:  HashSet<u8>,
    pub buttons_just_released: HashSet<u8>,
    pub axes:         [f32;6],
    pub rumble_low:   f32,
    pub rumble_high:  f32,
    pub battery_pct:  Option<u32>,
    pub kind:         GamepadKind,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum GamepadKind { Xbox, Playstation, Switch, Generic, Phone }

impl GamepadState {
    pub fn new(id:u32, name:&str) -> Self {
        Self { id, name:name.to_string(), connected:true, buttons_held:HashSet::new(),
               buttons_just_pressed:HashSet::new(), buttons_just_released:HashSet::new(),
               axes:[0.0;6], rumble_low:0.0, rumble_high:0.0, battery_pct:None, kind:GamepadKind::Generic }
    }
    pub fn axis(&self, a:GamepadAxis) -> f32 { self.axes[a as usize] }
    pub fn left_stick(&self) -> [f32;2] { [self.axes[0], self.axes[1]] }
    pub fn right_stick(&self) -> [f32;2] { [self.axes[2], self.axes[3]] }
    pub fn rumble(&mut self, low:f32, high:f32) { self.rumble_low=low.clamp(0.0,1.0); self.rumble_high=high.clamp(0.0,1.0); }
}

// ═══ TOUCH STATE ══════════════════════════════════════════════════
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct TouchState {
    pub touches: Vec<Touch>,
    pub gestures_this_frame: Vec<TouchGesture>,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Touch { pub id:u64, pub pos:[f32;2], pub delta:[f32;2], pub pressure:f32, pub phase:TouchPhase }
#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum TouchPhase { Began, Moved, Stationary, Ended, Cancelled }
impl TouchState {
    pub fn finger_count(&self) -> usize { self.touches.iter().filter(|t|t.phase!=TouchPhase::Ended&&t.phase!=TouchPhase::Cancelled).count() }
}

// ═══ PHONE CONTROLLER ════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct PhoneController {
    pub connected:     bool,
    pub device_name:   String,
    pub ip:            String,
    pub latency_ms:    u32,
    pub battery_pct:   u32,
    pub layout:        PhoneLayout,
    pub gyro_enabled:  bool,
    pub gyro:          [f32;3],   // roll, pitch, yaw
    pub accelero:      [f32;3],
    pub touch_joystick_l: [f32;2],
    pub touch_joystick_r: [f32;2],
    pub virtual_buttons: HashMap<String,bool>,
    pub connected_at:  Option<DateTime<Utc>>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum PhoneLayout { Gamepad, Dpad, Joystick, Drawing, Custom }

impl PhoneController {
    pub fn new() -> Self {
        Self { connected:false, device_name:String::new(), ip:String::new(), latency_ms:0,
               battery_pct:100, layout:PhoneLayout::Gamepad, gyro_enabled:true,
               gyro:[0.0;3], accelero:[0.0;3], touch_joystick_l:[0.0;2],
               touch_joystick_r:[0.0;2], virtual_buttons:HashMap::new(), connected_at:None }
    }
    pub fn tilt_x(&self) -> f32 { self.gyro[2] }  // yaw
    pub fn tilt_y(&self) -> f32 { self.gyro[1] }  // pitch
    pub fn virtual_btn(&self, name:&str) -> bool { *self.virtual_buttons.get(name).unwrap_or(&false) }
}

// ═══ VR INPUT ═════════════════════════════════════════════════════
#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct VrInput {
    pub enabled:      bool,
    pub hmd_position: [f32;3],
    pub hmd_rotation: [f32;4],
    pub hmd_vel:      [f32;3],
    pub left_hand:    VrHand,
    pub right_hand:   VrHand,
    pub eye_gaze:     Option<[f32;3]>,
    pub pass_through: bool,
}

#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct VrHand {
    pub position:  [f32;3],
    pub rotation:  [f32;4],
    pub velocity:  [f32;3],
    pub trigger:   f32,
    pub grip:      f32,
    pub joystick:  [f32;2],
    pub btn_a:     bool, pub btn_b:bool,
    pub btn_menu:  bool, pub btn_home:bool,
    pub gesture:   HandGesture,
    pub tracking:  TrackingState,
}

#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub enum HandGesture { #[default] None, Fist, OpenHand, Pointing, Pinch, ThumbsUp, Peace }
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub enum TrackingState { #[default] Good, Limited, Lost }

// ═══ INPUT MANAGER ════════════════════════════════════════════════
pub struct InputManager {
    pub keys_held:          HashSet<Key>,
    pub keys_just_pressed:  HashSet<Key>,
    pub keys_just_released: HashSet<Key>,
    pub mouse:              MouseState,
    pub gamepads:           HashMap<u32,GamepadState>,
    pub touch:              TouchState,
    pub phone:              PhoneController,
    pub vr:                 VrInput,
    pub actions:            HashMap<String,InputAction>,
    pub text_input:         String,
    pub text_input_active:  bool,
    pub frame:              u64,
    pub last_input_time:    Option<DateTime<Utc>>,
    pub haptics_enabled:    bool,
}

impl InputManager {
    pub fn new() -> Self {
        let mut m = Self {
            keys_held:HashSet::new(), keys_just_pressed:HashSet::new(),
            keys_just_released:HashSet::new(), mouse:MouseState::new(),
            gamepads:HashMap::new(), touch:TouchState::default(),
            phone:PhoneController::new(), vr:VrInput::default(),
            actions:HashMap::new(), text_input:String::new(),
            text_input_active:false, frame:0, last_input_time:None,
            haptics_enabled:true,
        };
        m.register_default_actions();
        m
    }

    fn register_default_actions(&mut self) {
        let actions = vec![
            InputAction::new("move_forward") .with_key(Key::W).with_key(Key::Up),
            InputAction::new("move_backward").with_key(Key::S).with_key(Key::Down),
            InputAction::new("move_left")    .with_key(Key::A).with_key(Key::Left),
            InputAction::new("move_right")   .with_key(Key::D).with_key(Key::Right),
            InputAction::new("jump")         .with_key(Key::Space).with_gamepad(0, GamepadButton::South),
            InputAction::new("sprint")       .with_key(Key::LShift).with_gamepad(0, GamepadButton::LThumb),
            InputAction::new("crouch")       .with_key(Key::LCtrl).with_gamepad(0, GamepadButton::RThumb),
            InputAction::new("interact")     .with_key(Key::E).with_gamepad(0, GamepadButton::West),
            InputAction::new("attack")       .with_mouse(MouseButton::Left).with_gamepad(0, GamepadButton::RTrigger),
            InputAction::new("aim")          .with_mouse(MouseButton::Right).with_gamepad(0, GamepadButton::LTrigger),
            InputAction::new("reload")       .with_key(Key::R).with_gamepad(0, GamepadButton::West),
            InputAction::new("dodge")        .with_key(Key::Q).with_gamepad(0, GamepadButton::East),
            InputAction::new("inventory")    .with_key(Key::I).with_gamepad(0, GamepadButton::Select),
            InputAction::new("map")          .with_key(Key::M).with_gamepad(0, GamepadButton::Home),
            InputAction::new("pause")        .with_key(Key::Escape).with_gamepad(0, GamepadButton::Start),
            InputAction::new("screenshot")   .with_key(Key::F12),
        ];
        for a in actions { self.actions.insert(a.name.clone(), a); }
    }

    pub fn begin_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse.buttons_just_pressed.clear();
        self.mouse.buttons_just_released.clear();
        self.mouse.delta = [0.0;2];
        self.mouse.scroll = [0.0;2];
        self.touch.gestures_this_frame.clear();
        for gp in self.gamepads.values_mut() { gp.buttons_just_pressed.clear(); gp.buttons_just_released.clear(); }
        self.frame += 1;
    }

    pub fn press_key(&mut self, k:Key) {
        if !self.keys_held.contains(&k) {
            self.keys_just_pressed.insert(k);
            self.last_input_time = Some(chrono::Utc::now());
        }
        self.keys_held.insert(k);
        if self.text_input_active { self.append_text_from_key(k); }
        self.update_actions();
    }

    pub fn release_key(&mut self, k:Key) {
        self.keys_held.remove(&k);
        self.keys_just_released.insert(k);
        self.update_actions();
    }

    fn append_text_from_key(&mut self, k:Key) {
        let ch = match k {
            Key::Space => Some(' '),
            Key::Backspace => { self.text_input.pop(); None },
            Key::A=>Some('a'),Key::B=>Some('b'),Key::C=>Some('c'),Key::D=>Some('d'),
            Key::E=>Some('e'),Key::F=>Some('f'),Key::G=>Some('g'),Key::H=>Some('h'),
            Key::I=>Some('i'),Key::J=>Some('j'),Key::K=>Some('k'),Key::L=>Some('l'),
            Key::M=>Some('m'),Key::N=>Some('n'),Key::O=>Some('o'),Key::P=>Some('p'),
            Key::Q=>Some('q'),Key::R=>Some('r'),Key::S=>Some('s'),Key::T=>Some('t'),
            Key::U=>Some('u'),Key::V=>Some('v'),Key::W=>Some('w'),Key::X=>Some('x'),
            Key::Y=>Some('y'),Key::Z=>Some('z'),
            Key::Num0=>Some('0'),Key::Num1=>Some('1'),Key::Num2=>Some('2'),
            Key::Num3=>Some('3'),Key::Num4=>Some('4'),Key::Num5=>Some('5'),
            Key::Num6=>Some('6'),Key::Num7=>Some('7'),Key::Num8=>Some('8'),Key::Num9=>Some('9'),
            _ => None,
        };
        if let Some(c) = ch { self.text_input.push(c); }
    }

    fn update_actions(&mut self) {
        let held = &self.keys_held;
        let just_pressed = &self.keys_just_pressed;
        let just_released = &self.keys_just_released;
        for action in self.actions.values_mut() {
            let was_held = action.held;
            action.held = action.bindings.iter().any(|b| match b {
                InputBinding::Key(k) => held.contains(k),
                _ => false,
            });
            action.just_pressed  = !was_held && action.held;
            action.just_released = was_held && !action.held;
            action.strength = if action.held { 1.0 } else { 0.0 };
        }
    }

    // ── Query API ──────────────────────────────────────────────────
    pub fn key(&self, k:Key)       -> bool { self.keys_held.contains(&k) }
    pub fn just_pressed(&self, k:Key)  -> bool { self.keys_just_pressed.contains(&k) }
    pub fn just_released(&self, k:Key) -> bool { self.keys_just_released.contains(&k) }

    pub fn action(&self, name:&str) -> bool { self.actions.get(name).map(|a|a.held).unwrap_or(false) }
    pub fn action_just_pressed(&self, name:&str) -> bool { self.actions.get(name).map(|a|a.just_pressed).unwrap_or(false) }
    pub fn action_strength(&self, name:&str) -> f32 { self.actions.get(name).map(|a|a.strength).unwrap_or(0.0) }

    pub fn move_dir(&self) -> [f32;2] {
        let x = if self.action("move_right") {1.0} else {0.0} - if self.action("move_left") {1.0} else {0.0};
        let y = if self.action("move_forward") {1.0} else {0.0} - if self.action("move_backward") {1.0} else {0.0};
        let len = (x*x+y*y).sqrt();
        if len > 0.0 { [x/len, y/len] } else { [0.0;2] }
    }

    pub fn rebind(&mut self, action:&str, binding:InputBinding) {
        if let Some(a) = self.actions.get_mut(action) {
            a.bindings.clear();
            a.bindings.push(binding);
            tracing::info!("Rebound action: {}", action);
        }
    }

    pub fn connect_gamepad(&mut self, id:u32, name:&str) {
        self.gamepads.insert(id, GamepadState::new(id, name));
        tracing::info!("Gamepad connected: {} ({})", name, id);
    }

    pub fn disconnect_gamepad(&mut self, id:u32) {
        self.gamepads.remove(&id);
        tracing::info!("Gamepad disconnected: {}", id);
    }

    pub fn any_input_this_frame(&self) -> bool {
        !self.keys_just_pressed.is_empty() || !self.mouse.buttons_just_pressed.is_empty()
    }

    pub fn time_since_last_input(&self) -> Option<f64> {
        self.last_input_time.map(|t| (chrono::Utc::now()-t).num_milliseconds() as f64/1000.0)
    }

    pub fn action_count(&self) -> usize { self.actions.len() }
    pub fn gamepad_count(&self) -> usize { self.gamepads.len() }
    pub fn phone_connected(&self) -> bool { self.phone.connected }
    pub fn vr_active(&self) -> bool { self.vr.enabled }
}

impl Default for InputManager { fn default() -> Self { Self::new() } }
extern crate tracing;
