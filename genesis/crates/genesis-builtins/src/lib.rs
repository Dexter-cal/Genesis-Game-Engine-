//! Genesis Built-in Script Functions & Standard Library
//!
//! Every scripting language in Genesis has access to these
//! built-in functions and objects. Like Godot's @GlobalScope,
//! these are available everywhere without imports.
//!
//! GLOBAL OBJECTS:
//!   world      — spawn/find/query entities
//!   player     — local player control
//!   self       — the entity this script is on
//!   game       — game state and flow
//!   scene      — scene management
//!   input      — input handling
//!   audio      — play sounds
//!   camera     — camera control
//!   ui         — HUD and menus
//!   ai         — AI agent access
//!   time       — game time
//!   math       — math utilities
//!   debug      — logging and debugging
//!   physics    — physics queries
//!   network    — multiplayer
//!   events     — custom events
//!   storage    — save/load data
//!   http       — web requests (sandboxed)
//!   studio     — studio features from script
//!   signal     — create/emit custom signals
//!
//! LIFECYCLE CALLBACKS:
//!   ready()           — called when node enters scene tree
//!   update(delta)     — called every frame
//!   physics_update(delta) — called at fixed rate
//!   on_signal(name, args) — respond to signals
//!
//! SIGNAL DECORATORS:
//!   @ready            — runs on ready()
//!   @update           — runs every frame
//!   @physics          — runs at physics rate
//!   @on("signal")     — connects to signal
//!   @input("action")  — responds to input action
//!   @timeout(secs)    — single-shot timer
//!   @interval(secs)   — repeating timer
//!   @tool             — also runs in editor

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Documentation for a built-in function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinDoc {
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub signature: String,
    pub parameters: Vec<ParamDoc>,
    pub returns: String,
    pub example: String,
    pub since_version: String,
    pub deprecated: bool,
    pub deprecated_message: Option<String>,
    pub category: ApiCategory,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDoc {
    pub name: String,
    pub type_: String,
    pub default: Option<String>,
    pub description: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiCategory {
    Entity, Transform, Physics, Rendering, Audio, Input,
    Camera, Ui, Ai, Network, Storage, Math, Debug,
    Signal, Timer, Scene, Game, Animation, Time,
}

/// The complete Genesis built-in API surface
pub struct BuiltinApi {
    pub functions: Vec<BuiltinDoc>,
    pub classes: Vec<BuiltinClass>,
    pub constants: Vec<BuiltinConstant>,
    pub decorators: Vec<DecoratorDoc>,
    pub signals: Vec<SignalDoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinClass {
    pub name: String,
    pub description: String,
    pub properties: Vec<PropertyDoc>,
    pub methods: Vec<BuiltinDoc>,
    pub signals: Vec<SignalDoc>,
    pub extends: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDoc {
    pub name: String,
    pub type_: String,
    pub description: String,
    pub readable: bool,
    pub writable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltinConstant {
    pub name: String,
    pub type_: String,
    pub value: String,
    pub description: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoratorDoc {
    pub name: String,
    pub description: String,
    pub syntax: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDoc {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParamDoc>,
    pub emitted_by: Vec<String>,
}

impl BuiltinApi {
    pub fn full() -> Self {
        let mut api = Self {
            functions: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
            decorators: Vec::new(),
            signals: Vec::new(),
        };
        api.register_world_api();
        api.register_self_api();
        api.register_player_api();
        api.register_audio_api();
        api.register_input_api();
        api.register_camera_api();
        api.register_physics_api();
        api.register_math_api();
        api.register_ui_api();
        api.register_debug_api();
        api.register_time_api();
        api.register_network_api();
        api.register_storage_api();
        api.register_ai_api();
        api.register_animation_api();
        api.register_signal_api();
        api.register_scene_api();
        api.register_game_api();
        api.register_decorators();
        api.register_constants();
        api
    }

    fn f(name: &str, ns: &str, sig: &str, desc: &str, returns: &str, example: &str, cat: ApiCategory) -> BuiltinDoc {
        BuiltinDoc {
            name: name.to_string(), namespace: ns.to_string(),
            description: desc.to_string(), signature: sig.to_string(),
            parameters: Vec::new(), returns: returns.to_string(),
            example: example.to_string(), since_version: "0.1.0".to_string(),
            deprecated: false, deprecated_message: None, category: cat, tags: Vec::new(),
        }
    }

    fn register_world_api(&mut self) {
        let fns = vec![
            Self::f("spawn", "world", "spawn(type: str, name: str, pos: Vec3) -> Entity",
                "Spawn a new entity in the world",
                "Entity",
                "let e = world.spawn(\"npc\", \"Guard\", Vec3(10, 0, 5))",
                ApiCategory::Entity),
            Self::f("spawn_at", "world", "spawn_at(scene_path: str, pos: Vec3) -> Entity",
                "Instantiate a scene at a position",
                "Entity",
                "let enemy = world.spawn_at(\"res://enemies/goblin.scene\", [0,0,0])",
                ApiCategory::Entity),
            Self::f("find", "world", "find(id: str) -> Entity?",
                "Find an entity by ID or name",
                "Entity?",
                "let boss = world.find(\"boss_dragon\")",
                ApiCategory::Entity),
            Self::f("find_all_with_tag", "world", "find_all_with_tag(tag: str) -> [Entity]",
                "Find all entities with a specific tag",
                "[Entity]",
                "let enemies = world.find_all_with_tag(\"enemy\")\nfor e in enemies: e.take_damage(10)",
                ApiCategory::Entity),
            Self::f("entities_near", "world", "entities_near(pos: Vec3, radius: f32) -> [Entity]",
                "Find all entities within radius of a position",
                "[Entity]",
                "let nearby = world.entities_near(self.position, 10.0)",
                ApiCategory::Entity),
            Self::f("entities_in_group", "world", "entities_in_group(group: str) -> [Entity]",
                "Find all entities in a named group",
                "[Entity]",
                "let guards = world.entities_in_group(\"guard_patrol\")",
                ApiCategory::Entity),
            Self::f("destroy", "world", "destroy(entity: Entity)",
                "Remove an entity from the world",
                "void",
                "world.destroy(old_barrel)",
                ApiCategory::Entity),
            Self::f("raycast", "world", "raycast(from: Vec3, dir: Vec3, max_dist: f32) -> RayHit?",
                "Cast a ray and return the first hit",
                "RayHit?",
                "if let hit = world.raycast(pos, forward, 50.0):\n  hit.entity.take_damage(25)",
                ApiCategory::Physics),
            Self::f("get_time", "world", "get_time() -> f32",
                "Get current game time in hours (0-24)",
                "f32",
                "if world.get_time() > 20.0: spawn_night_enemies()",
                ApiCategory::Time),
            Self::f("get_weather", "world", "get_weather() -> str",
                "Get current weather type as string",
                "str",
                "if world.get_weather() == \"rain\": play_rain_sound()",
                ApiCategory::Game),
            Self::f("set_weather", "world", "set_weather(type: str, intensity: f32)",
                "Change the weather",
                "void",
                "world.set_weather(\"thunderstorm\", 0.8)",
                ApiCategory::Game),
            Self::f("get_flag", "world", "get_flag(name: str) -> bool",
                "Get a global world flag",
                "bool",
                "if world.get_flag(\"boss_defeated\"): unlock_next_area()",
                ApiCategory::Game),
            Self::f("set_flag", "world", "set_flag(name: str, value: bool)",
                "Set a global world flag",
                "void",
                "world.set_flag(\"merchant_met\", true)",
                ApiCategory::Game),
        ];
        self.functions.extend(fns);
    }

    fn register_self_api(&mut self) {
        let fns = vec![
            // Position & Transform
            Self::f("move_to", "self", "move_to(target: Vec3, speed: f32?)",
                "Move this entity toward a position (uses NavAgent if available)",
                "void",
                "self.move_to(player.position, 3.5)",
                ApiCategory::Transform),
            Self::f("look_at", "self", "look_at(target: Vec3 | Entity)",
                "Rotate to face a target",
                "void",
                "self.look_at(player)",
                ApiCategory::Transform),
            Self::f("teleport", "self", "teleport(pos: Vec3)",
                "Instantly move to position",
                "void",
                "self.teleport(spawn_point.position)",
                ApiCategory::Transform),
            // NPC actions
            Self::f("say", "self", "say(text: str, emotion: str?)",
                "Speak a line of dialogue",
                "void",
                "self.say(\"Who goes there?\", \"suspicious\")",
                ApiCategory::Ai),
            Self::f("play_animation", "self", "play_animation(name: str, speed: f32?)",
                "Play an animation",
                "void",
                "self.play_animation(\"attack_swing\", 1.5)",
                ApiCategory::Animation),
            Self::f("take_damage", "self", "take_damage(amount: f32, type: str?)",
                "Apply damage to this entity",
                "f32",
                "let actual = self.take_damage(25, \"fire\")",
                ApiCategory::Entity),
            Self::f("heal", "self", "heal(amount: f32)",
                "Heal this entity",
                "f32",
                "self.heal(50)",
                ApiCategory::Entity),
            Self::f("is_alive", "self", "is_alive() -> bool",
                "Check if entity is alive (health > 0)",
                "bool",
                "if not self.is_alive(): drop_loot()",
                ApiCategory::Entity),
            Self::f("get_tag", "self", "get_tag(tag: str) -> bool",
                "Check if entity has a tag",
                "bool",
                "if self.get_tag(\"flying\"): disable_gravity()",
                ApiCategory::Entity),
            Self::f("emit", "self", "emit(signal: str, ...args)",
                "Emit a custom signal",
                "void",
                "self.emit(\"door_opened\", player_id)",
                ApiCategory::Signal),
            Self::f("set_visible", "self", "set_visible(visible: bool)",
                "Show or hide this entity",
                "void",
                "self.set_visible(false)",
                ApiCategory::Rendering),
            Self::f("set_material", "self", "set_material(material_id: str, slot: i32?)",
                "Change the material",
                "void",
                "self.set_material(\"highlight_red\", 0)",
                ApiCategory::Rendering),
            Self::f("spawn_effect", "self", "spawn_effect(effect_id: str, offset: Vec3?)",
                "Spawn a particle effect at this entity",
                "void",
                "self.spawn_effect(\"blood_splatter\")",
                ApiCategory::Rendering),
        ];
        self.functions.extend(fns);
    }

    fn register_player_api(&mut self) {
        let fns = vec![
            Self::f("give_item", "player", "give_item(item_id: str, count: i32?)",
                "Add item to player inventory",
                "void",
                "player.give_item(\"health_potion\", 3)",
                ApiCategory::Entity),
            Self::f("take_item", "player", "take_item(item_id: str, count: i32?) -> bool",
                "Remove item from inventory. Returns false if not enough.",
                "bool",
                "if player.take_item(\"gold_coin\", 100): unlock_shop()",
                ApiCategory::Entity),
            Self::f("has_item", "player", "has_item(item_id: str, count: i32?) -> bool",
                "Check if player has an item",
                "bool",
                "if player.has_item(\"magic_key\"): open_door()",
                ApiCategory::Entity),
            Self::f("add_gold", "player", "add_gold(amount: i64)",
                "Add gold to player",
                "void",
                "player.add_gold(500)",
                ApiCategory::Entity),
            Self::f("get_gold", "player", "get_gold() -> i64",
                "Get player gold amount",
                "i64",
                "ui.show(\"Gold: \" + str(player.get_gold()))",
                ApiCategory::Entity),
            Self::f("add_xp", "player", "add_xp(amount: i64)",
                "Give experience points to player",
                "void",
                "player.add_xp(1000)",
                ApiCategory::Entity),
            Self::f("get_level", "player", "get_level() -> i32",
                "Get player level",
                "i32",
                "if player.get_level() >= 5: unlock_ability(\"double_jump\")",
                ApiCategory::Entity),
            Self::f("start_quest", "player", "start_quest(quest_id: str)",
                "Start a quest for the player",
                "void",
                "player.start_quest(\"main_quest_01\")",
                ApiCategory::Game),
            Self::f("complete_quest", "player", "complete_quest(quest_id: str)",
                "Complete a quest",
                "void",
                "player.complete_quest(\"find_the_sword\")",
                ApiCategory::Game),
            Self::f("show_notification", "player", "show_notification(text: str, icon: str?, duration: f32?)",
                "Show a notification to the player",
                "void",
                "player.show_notification(\"New Area Discovered!\", \"icon_map\", 3.0)",
                ApiCategory::Ui),
        ];
        self.functions.extend(fns);
    }

    fn register_audio_api(&mut self) {
        let fns = vec![
            Self::f("play", "audio", "play(clip_id: str, volume: f32?, pitch: f32?) -> AudioHandle",
                "Play a sound globally",
                "AudioHandle",
                "audio.play(\"victory_fanfare\")",
                ApiCategory::Audio),
            Self::f("play_at", "audio", "play_at(clip_id: str, pos: Vec3, range: f32?) -> AudioHandle",
                "Play a spatial sound at a world position",
                "AudioHandle",
                "audio.play_at(\"explosion_large\", self.position, 50.0)",
                ApiCategory::Audio),
            Self::f("stop", "audio", "stop(handle: AudioHandle)",
                "Stop a playing sound",
                "void",
                "audio.stop(my_sound_handle)",
                ApiCategory::Audio),
            Self::f("play_music", "audio", "play_music(track_id: str, fade: f32?)",
                "Play a music track",
                "void",
                "audio.play_music(\"boss_theme\", 2.0)",
                ApiCategory::Audio),
            Self::f("stop_music", "audio", "stop_music(fade: f32?)",
                "Stop the current music",
                "void",
                "audio.stop_music(1.5)",
                ApiCategory::Audio),
            Self::f("set_music_intensity", "audio", "set_music_intensity(value: f32)",
                "Set adaptive music intensity (0-1)",
                "void",
                "audio.set_music_intensity(0.9)  # intense combat",
                ApiCategory::Audio),
            Self::f("set_volume", "audio", "set_volume(bus: str, volume: f32)",
                "Set volume for an audio bus",
                "void",
                "audio.set_volume(\"sfx\", 0.8)",
                ApiCategory::Audio),
        ];
        self.functions.extend(fns);
    }

    fn register_input_api(&mut self) {
        let fns = vec![
            Self::f("is_pressed", "input", "is_pressed(action: str) -> bool",
                "Check if an action is currently held",
                "bool",
                "if input.is_pressed(\"jump\"): apply_jump_force()",
                ApiCategory::Input),
            Self::f("just_pressed", "input", "just_pressed(action: str) -> bool",
                "True only on the frame an action was pressed",
                "bool",
                "if input.just_pressed(\"attack\"): play_attack_anim()",
                ApiCategory::Input),
            Self::f("just_released", "input", "just_released(action: str) -> bool",
                "True only on the frame an action was released",
                "bool",
                "if input.just_released(\"charge\"): fire_charged_shot()",
                ApiCategory::Input),
            Self::f("get_axis", "input", "get_axis(neg: str, pos: str) -> f32",
                "Get axis value (-1 to 1) from two actions",
                "f32",
                "let move_x = input.get_axis(\"move_left\", \"move_right\")",
                ApiCategory::Input),
            Self::f("get_vector", "input", "get_vector(left: str, right: str, up: str, down: str) -> Vec2",
                "Get 2D movement vector from 4 actions",
                "Vec2",
                "let dir = input.get_vector(\"move_left\", \"move_right\", \"move_forward\", \"move_back\")",
                ApiCategory::Input),
            Self::f("rumble", "input", "rumble(player: i32, strong: f32, weak: f32, duration: f32)",
                "Rumble a player's controller",
                "void",
                "input.rumble(0, 1.0, 0.5, 0.3)",
                ApiCategory::Input),
        ];
        self.functions.extend(fns);
    }

    fn register_camera_api(&mut self) {
        let fns = vec![
            Self::f("set_mode", "camera", "set_mode(mode: str)",
                "Switch camera mode (first_person, third_close, third_far, drone, strategy, orbit, window_3d)",
                "void",
                "camera.set_mode(\"first_person\")",
                ApiCategory::Camera),
            Self::f("shake", "camera", "shake(trauma: f32)",
                "Add camera shake (0-1 trauma amount)",
                "void",
                "camera.shake(0.8)  # big explosion",
                ApiCategory::Camera),
            Self::f("set_fov", "camera", "set_fov(fov: f32, duration: f32?)",
                "Set camera field of view (optionally smoothly)",
                "void",
                "camera.set_fov(40.0, 0.5)  # scope zoom",
                ApiCategory::Camera),
            Self::f("enable_3d_window", "camera", "enable_3d_window(screen_w: f32, screen_h: f32, dist: f32)",
                "Enable 3D window / off-axis head-tracked mode",
                "void",
                "camera.enable_3d_window(52.0, 29.0, 60.0)  # 24\" screen at 60cm",
                ApiCategory::Camera),
            Self::f("transition_to", "camera", "transition_to(cam_id: str, type: str, duration: f32)",
                "Transition to another camera",
                "void",
                "camera.transition_to(\"boss_intro_cam\", \"fade_black\", 1.0)",
                ApiCategory::Camera),
        ];
        self.functions.extend(fns);
    }

    fn register_physics_api(&mut self) {
        let fns = vec![
            Self::f("raycast", "physics", "raycast(from: Vec3, dir: Vec3, dist: f32, mask: i32?) -> RayHit?",
                "Cast a ray against the physics world",
                "RayHit?",
                "let hit = physics.raycast(pos, Vec3.DOWN, 10.0)\nif hit: is_grounded = true",
                ApiCategory::Physics),
            Self::f("sphere_cast", "physics", "sphere_cast(from: Vec3, to: Vec3, radius: f32) -> [RayHit]",
                "Cast a sphere and return all hits",
                "[RayHit]",
                "let hits = physics.sphere_cast(self.position, target, 2.0)",
                ApiCategory::Physics),
            Self::f("overlap_sphere", "physics", "overlap_sphere(center: Vec3, radius: f32, mask: i32?) -> [Entity]",
                "Find all physics bodies overlapping a sphere",
                "[Entity]",
                "let in_blast = physics.overlap_sphere(explosion_pos, 15.0)\nfor e in in_blast: e.take_damage(75)",
                ApiCategory::Physics),
            Self::f("apply_impulse", "physics", "apply_impulse(entity: Entity, force: Vec3)",
                "Apply a physics impulse to an entity",
                "void",
                "physics.apply_impulse(barrel, Vec3(0, 500, 200))",
                ApiCategory::Physics),
        ];
        self.functions.extend(fns);
    }

    fn register_math_api(&mut self) {
        let fns = vec![
            Self::f("lerp",  "math", "lerp(a: f32, b: f32, t: f32) -> f32", "Linear interpolation", "f32", "let blend = math.lerp(0.0, 1.0, progress)", ApiCategory::Math),
            Self::f("clamp", "math", "clamp(v: f32, min: f32, max: f32) -> f32", "Clamp value", "f32", "speed = math.clamp(speed, 0.0, max_speed)", ApiCategory::Math),
            Self::f("rand",  "math", "rand(min: f32, max: f32) -> f32", "Random float in range", "f32", "let delay = math.rand(0.5, 2.0)", ApiCategory::Math),
            Self::f("rand_int", "math", "rand_int(min: i32, max: i32) -> i32", "Random integer", "i32", "let item = loot_table[math.rand_int(0, len(loot_table)-1)]", ApiCategory::Math),
            Self::f("distance", "math", "distance(a: Vec3, b: Vec3) -> f32", "3D distance", "f32", "let d = math.distance(self.position, player.position)", ApiCategory::Math),
            Self::f("angle_to", "math", "angle_to(from: Vec3, to: Vec3) -> f32", "Angle between vectors (radians)", "f32", "let a = math.angle_to(forward, target_dir)", ApiCategory::Math),
            Self::f("normalize", "math", "normalize(v: Vec3) -> Vec3", "Normalize vector to unit length", "Vec3", "let dir = math.normalize(target - self.position)", ApiCategory::Math),
            Self::f("smoothstep", "math", "smoothstep(a: f32, b: f32, t: f32) -> f32", "Smooth interpolation", "f32", "let blend = math.smoothstep(0, 1, progress)", ApiCategory::Math),
            Self::f("sin", "math", "sin(x: f32) -> f32", "Sine", "f32", "let wave = math.sin(time * 2.0) * amplitude", ApiCategory::Math),
            Self::f("cos", "math", "cos(x: f32) -> f32", "Cosine", "f32", "let x = math.cos(angle) * radius", ApiCategory::Math),
        ];
        self.functions.extend(fns);
    }

    fn register_ui_api(&mut self) {
        let fns = vec![
            Self::f("show_message", "ui", "show_message(text: str, duration: f32?)", "Show screen message", "void", "ui.show_message(\"Quest Complete!\", 3.0)", ApiCategory::Ui),
            Self::f("show_dialogue", "ui", "show_dialogue(speaker: str, text: str, options: [str]?) -> i32?", "Show dialogue box, returns chosen option index", "i32?", "let choice = ui.show_dialogue(\"Elder\", \"Choose wisely...\", [\"Yes\", \"No\"])", ApiCategory::Ui),
            Self::f("fade_screen", "ui", "fade_screen(to_black: bool, duration: f32)", "Fade to/from black", "void", "ui.fade_screen(true, 1.0)", ApiCategory::Ui),
            Self::f("show_minimap_marker", "ui", "show_minimap_marker(pos: Vec3, icon: str, color: Color?)", "Add minimap marker", "str", "let id = ui.show_minimap_marker(treasure.position, \"icon_chest\")", ApiCategory::Ui),
            Self::f("remove_minimap_marker", "ui", "remove_minimap_marker(id: str)", "Remove minimap marker", "void", "ui.remove_minimap_marker(marker_id)", ApiCategory::Ui),
            Self::f("show_hud_element", "ui", "show_hud_element(element_id: str, visible: bool)", "Show/hide HUD element", "void", "ui.show_hud_element(\"crosshair\", false)", ApiCategory::Ui),
        ];
        self.functions.extend(fns);
    }

    fn register_debug_api(&mut self) {
        let fns = vec![
            Self::f("log",   "debug", "log(msg: str, ...args)", "Print to console", "void", "debug.log(\"Health:\", self.health)", ApiCategory::Debug),
            Self::f("warn",  "debug", "warn(msg: str)", "Warning to console", "void", "debug.warn(\"Path not found\")", ApiCategory::Debug),
            Self::f("error", "debug", "error(msg: str)", "Error to console", "void", "debug.error(\"NPC has no target!\")", ApiCategory::Debug),
            Self::f("draw_line",   "debug", "draw_line(from: Vec3, to: Vec3, color: Color?)", "Draw debug line", "void", "debug.draw_line(self.position, target, Color.RED)", ApiCategory::Debug),
            Self::f("draw_sphere", "debug", "draw_sphere(center: Vec3, radius: f32, color: Color?)", "Draw debug sphere", "void", "debug.draw_sphere(self.position, detection_range, Color.YELLOW)", ApiCategory::Debug),
            Self::f("print_fps", "debug", "print_fps()", "Log current FPS", "void", "debug.print_fps()", ApiCategory::Debug),
        ];
        self.functions.extend(fns);
    }

    fn register_time_api(&mut self) {
        let fns = vec![
            Self::f("wait", "time", "wait(seconds: f32)", "Pause script execution (async)", "void", "self.say(\"Goodbye\")\nawait time.wait(1.0)\nself.walk_away()", ApiCategory::Time),
            Self::f("get_delta", "time", "get_delta() -> f32", "Get current frame delta time", "f32", "pos += velocity * time.get_delta()", ApiCategory::Time),
            Self::f("get_elapsed", "time", "get_elapsed() -> f32", "Seconds since game started", "f32", "let wave = sin(time.get_elapsed() * freq)", ApiCategory::Time),
            Self::f("game_time_hour", "time", "game_time_hour() -> f32", "In-game time (0-24)", "f32", "if time.game_time_hour() < 6: set_night_mode()", ApiCategory::Time),
            Self::f("create_timer", "time", "create_timer(secs: f32, repeat: bool?) -> TimerHandle", "Create a timer", "TimerHandle", "let t = time.create_timer(5.0)\nawait t.timeout", ApiCategory::Time),
        ];
        self.functions.extend(fns);
    }

    fn register_network_api(&mut self) {
        let fns = vec![
            Self::f("is_server", "network", "is_server() -> bool", "True if this is the authoritative server", "bool", "if network.is_server(): spawn_boss()", ApiCategory::Network),
            Self::f("get_players", "network", "get_players() -> [Player]", "Get all connected players", "[Player]", "for p in network.get_players(): show_player_tag(p)", ApiCategory::Network),
            Self::f("rpc", "network", "rpc(func: str, ...args)", "Call a function on all clients", "void", "network.rpc(\"show_explosion\", position)", ApiCategory::Network),
            Self::f("rpc_id", "network", "rpc_id(peer: i32, func: str, ...args)", "Call function on specific client", "void", "network.rpc_id(player.id, \"give_reward\", gold)", ApiCategory::Network),
            Self::f("broadcast_event", "network", "broadcast_event(event: str, data)", "Broadcast game event to all players", "void", "network.broadcast_event(\"boss_died\", {\"boss_id\": boss.id})", ApiCategory::Network),
        ];
        self.functions.extend(fns);
    }

    fn register_storage_api(&mut self) {
        let fns = vec![
            Self::f("save", "storage", "save(key: str, value)", "Save data to persistent storage", "void", "storage.save(\"player_health\", health)", ApiCategory::Entity),
            Self::f("load", "storage", "load(key: str, default?) -> any", "Load data from storage", "any", "health = storage.load(\"player_health\", 100)", ApiCategory::Entity),
            Self::f("has", "storage", "has(key: str) -> bool", "Check if a key exists in storage", "bool", "if not storage.has(\"tutorial_done\"): show_tutorial()", ApiCategory::Entity),
            Self::f("delete", "storage", "delete(key: str)", "Delete a storage key", "void", "storage.delete(\"temp_data\")", ApiCategory::Entity),
            Self::f("save_game", "storage", "save_game(slot: i32)", "Save entire game state to a slot", "void", "storage.save_game(1)", ApiCategory::Game),
            Self::f("load_game", "storage", "load_game(slot: i32) -> bool", "Load game state from slot", "bool", "if storage.load_game(1): fade_in()", ApiCategory::Game),
        ];
        self.functions.extend(fns);
    }

    fn register_ai_api(&mut self) {
        let fns = vec![
            Self::f("ask", "ai", "ask(prompt: str, context: str?) -> str", "Send a prompt to AI and get response", "str", "let response = await ai.ask(\"What should the guard say to a suspicious traveler?\")", ApiCategory::Ai),
            Self::f("npc_decide", "ai", "npc_decide(npc_id: str, context) -> str", "Get AI decision for an NPC", "str", "let action = await ai.npc_decide(self.id, {\"player_near\": true})", ApiCategory::Ai),
            Self::f("generate_dialogue", "ai", "generate_dialogue(npc_id: str, player_input: str) -> str", "Generate NPC dialogue", "str", "let reply = await ai.generate_dialogue(self.id, player.last_said)", ApiCategory::Ai),
            Self::f("suggest_item", "ai", "suggest_item(player_data, shop_data) -> [str]", "AI item recommendations", "[str]", "let recs = await ai.suggest_item(player.stats, shop.inventory)", ApiCategory::Ai),
            Self::f("generate_quest", "ai", "generate_quest(context) -> QuestData", "Generate a quest from context", "QuestData", "let quest = await ai.generate_quest({\"location\": \"forest\", \"level\": 5})", ApiCategory::Ai),
        ];
        self.functions.extend(fns);
    }

    fn register_animation_api(&mut self) {
        let fns = vec![
            Self::f("play", "anim", "play(name: str, speed: f32?, blend: f32?)", "Play animation", "void", "anim.play(\"walk\", 1.2)", ApiCategory::Animation),
            Self::f("play_blend", "anim", "play_blend(from: str, to: str, t: f32)", "Blend between two animations", "void", "anim.play_blend(\"walk\", \"run\", sprint_factor)", ApiCategory::Animation),
            Self::f("set_param", "anim", "set_param(name: str, value)", "Set animation tree parameter", "void", "anim.set_param(\"speed\", velocity.length())\nanim.set_param(\"is_grounded\", on_floor)", ApiCategory::Animation),
            Self::f("trigger", "anim", "trigger(name: str)", "Fire animation trigger", "void", "anim.trigger(\"attack\")", ApiCategory::Animation),
        ];
        self.functions.extend(fns);
    }

    fn register_signal_api(&mut self) {
        let fns = vec![
            Self::f("emit", "signal", "emit(name: str, ...args)", "Emit a signal", "void", "signal.emit(\"item_collected\", item_id, player.id)", ApiCategory::Signal),
            Self::f("connect", "signal", "connect(node: Entity, sig: str, target: Entity, method: str)", "Connect a signal", "void", "signal.connect(door, \"opened\", guard, \"alert\")", ApiCategory::Signal),
            Self::f("disconnect", "signal", "disconnect(node: Entity, sig: str, target: Entity)", "Disconnect a signal", "void", "signal.disconnect(alarm, \"triggered\", self)", ApiCategory::Signal),
        ];
        self.functions.extend(fns);
    }

    fn register_scene_api(&mut self) {
        let fns = vec![
            Self::f("load", "scene", "load(path: str)", "Load a scene", "void", "scene.load(\"res://scenes/village.scene\")", ApiCategory::Scene),
            Self::f("add", "scene", "add(scene_path: str, parent: Entity?) -> Entity", "Add a scene instance", "Entity", "let chest = scene.add(\"res://props/chest.scene\", room)", ApiCategory::Scene),
            Self::f("get_node", "scene", "get_node(path: str) -> Entity?", "Get node by scene path", "Entity?", "let boss_arena = scene.get_node(\"/root/dungeon/boss_room\")", ApiCategory::Scene),
        ];
        self.functions.extend(fns);
    }

    fn register_game_api(&mut self) {
        let fns = vec![
            Self::f("quit", "game", "quit()", "Exit the game", "void", "if input.just_pressed(\"escape\"): game.quit()", ApiCategory::Game),
            Self::f("pause", "game", "pause(paused: bool)", "Pause/unpause the game", "void", "game.pause(true)", ApiCategory::Game),
            Self::f("restart", "game", "restart()", "Restart the current scene", "void", "game.restart()", ApiCategory::Game),
            Self::f("set_timescale", "game", "set_timescale(scale: f32)", "Set game time scale (1.0 = normal, 0.5 = slow, 2.0 = fast)", "void", "game.set_timescale(0.3)  # bullet time", ApiCategory::Game),
        ];
        self.functions.extend(fns);
    }

    fn register_decorators(&mut self) {
        self.decorators = vec![
            DecoratorDoc { name: "@ready".to_string(), description: "Called when node enters scene tree. Use for initialization.".to_string(), syntax: "@ready\ndef on_ready():".to_string(), example: "@ready\ndef on_ready():\n    self.play_animation(\"idle\")".to_string() },
            DecoratorDoc { name: "@update".to_string(), description: "Called every frame. Receives delta time.".to_string(), syntax: "@update\ndef on_update(delta: f32):".to_string(), example: "@update\ndef on_update(delta: f32):\n    self.position.y += sin(time.elapsed) * 0.1".to_string() },
            DecoratorDoc { name: "@physics".to_string(), description: "Called at fixed physics rate. Use for movement and forces.".to_string(), syntax: "@physics\ndef on_physics(delta: f32):".to_string(), example: "@physics\ndef on_physics(delta: f32):\n    velocity += gravity * delta".to_string() },
            DecoratorDoc { name: "@on".to_string(), description: "Connect to a signal. Called when that signal fires.".to_string(), syntax: "@on(\"signal_name\")\ndef handler(...args):".to_string(), example: "@on(\"body_entered\")\ndef on_body_entered(body: Entity):\n    if body.get_tag(\"player\"):\n        player.take_damage(10)".to_string() },
            DecoratorDoc { name: "@input".to_string(), description: "Called when an input action fires.".to_string(), syntax: "@input(\"action_name\")\ndef on_action():".to_string(), example: "@input(\"interact\")\ndef on_interact():\n    if player_nearby: start_dialogue()".to_string() },
            DecoratorDoc { name: "@timeout".to_string(), description: "Called once after N seconds.".to_string(), syntax: "@timeout(seconds: f32)\ndef on_timeout():".to_string(), example: "@timeout(5.0)\ndef self_destruct():\n    world.destroy(self)".to_string() },
            DecoratorDoc { name: "@interval".to_string(), description: "Called repeatedly every N seconds.".to_string(), syntax: "@interval(seconds: f32)\ndef on_tick():".to_string(), example: "@interval(2.0)\ndef check_patrol():\n    if not enemy_near: resume_patrol()".to_string() },
            DecoratorDoc { name: "@tool".to_string(), description: "Script also runs in editor. Use for editor tools.".to_string(), syntax: "@tool".to_string(), example: "@tool\n# Script runs in editor for live preview".to_string() },
        ];
    }

    fn register_constants(&mut self) {
        self.constants = vec![
            BuiltinConstant { name: "PI".to_string(), type_: "f32".to_string(), value: "3.14159265".to_string(), description: "Pi constant".to_string(), namespace: "math".to_string() },
            BuiltinConstant { name: "TAU".to_string(), type_: "f32".to_string(), value: "6.28318530".to_string(), description: "Two pi".to_string(), namespace: "math".to_string() },
            BuiltinConstant { name: "DEG2RAD".to_string(), type_: "f32".to_string(), value: "0.01745329".to_string(), description: "Multiply degrees by this to get radians".to_string(), namespace: "math".to_string() },
            BuiltinConstant { name: "RAD2DEG".to_string(), type_: "f32".to_string(), value: "57.29577951".to_string(), description: "Multiply radians by this to get degrees".to_string(), namespace: "math".to_string() },
            BuiltinConstant { name: "INF".to_string(), type_: "f32".to_string(), value: "infinity".to_string(), description: "Positive infinity".to_string(), namespace: "math".to_string() },
            BuiltinConstant { name: "UP".to_string(), type_: "Vec3".to_string(), value: "Vec3(0, 1, 0)".to_string(), description: "World up direction".to_string(), namespace: "Vec3".to_string() },
            BuiltinConstant { name: "DOWN".to_string(), type_: "Vec3".to_string(), value: "Vec3(0, -1, 0)".to_string(), description: "World down direction".to_string(), namespace: "Vec3".to_string() },
            BuiltinConstant { name: "FORWARD".to_string(), type_: "Vec3".to_string(), value: "Vec3(0, 0, -1)".to_string(), description: "World forward direction".to_string(), namespace: "Vec3".to_string() },
            BuiltinConstant { name: "RIGHT".to_string(), type_: "Vec3".to_string(), value: "Vec3(1, 0, 0)".to_string(), description: "World right direction".to_string(), namespace: "Vec3".to_string() },
            BuiltinConstant { name: "ZERO".to_string(), type_: "Vec3".to_string(), value: "Vec3(0, 0, 0)".to_string(), description: "Zero vector".to_string(), namespace: "Vec3".to_string() },
        ];
    }

    pub fn search(&self, query: &str) -> Vec<&BuiltinDoc> {
        let q = query.to_lowercase();
        self.functions.iter()
            .filter(|f| f.name.to_lowercase().contains(&q) || f.description.to_lowercase().contains(&q) || f.namespace.to_lowercase().contains(&q))
            .collect()
    }

    pub fn get_namespace(&self, ns: &str) -> Vec<&BuiltinDoc> {
        self.functions.iter().filter(|f| f.namespace == ns).collect()
    }

    pub fn total_api_surface(&self) -> usize {
        self.functions.len() + self.constants.len() + self.decorators.len()
    }
}
