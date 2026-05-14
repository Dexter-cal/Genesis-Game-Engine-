//! Project Templates — starter kits for every game genre
use serde::{Serialize,Deserialize};
use std::collections::HashMap;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ProjectTemplate {
    pub id:          String,
    pub name:        String,
    pub description: String,
    pub genre:       Genre,
    pub preview_img: String,
    pub scenes:      Vec<TemplateScene>,
    pub scripts:     Vec<TemplateScript>,
    pub assets:      Vec<AssetRef>,
    pub features:    Vec<String>,
    pub difficulty:  SetupDifficulty,
    pub tags:        Vec<String>,
    pub ai_prompt:   String,   // sent to AI to help customise on creation
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Genre {
    Blank, Fps, Rpg, Platformer2D, Platformer3D,
    TopDown, Racing, Strategy, Puzzle, Horror,
    OpenWorld, VisualNovel, SpaceShooter,
    BattleRoyale, Survival, TowerDefense,
    Sports, Rhythm, Roguelike, Metroidvania,
    PointAndClick, SimCity, Custom(String),
}

impl Genre {
    pub fn name(&self) -> &str {
        match self {
            Self::Blank=>"Blank",Self::Fps=>"FPS",Self::Rpg=>"RPG",
            Self::Platformer2D=>"2D Platformer",Self::Platformer3D=>"3D Platformer",
            Self::TopDown=>"Top-Down",Self::Racing=>"Racing",Self::Strategy=>"Strategy",
            Self::Puzzle=>"Puzzle",Self::Horror=>"Horror",Self::OpenWorld=>"Open World",
            Self::VisualNovel=>"Visual Novel",Self::SpaceShooter=>"Space Shooter",
            Self::BattleRoyale=>"Battle Royale",Self::Survival=>"Survival",
            Self::TowerDefense=>"Tower Defense",Self::Sports=>"Sports",
            Self::Rhythm=>"Rhythm",Self::Roguelike=>"Roguelike",
            Self::Metroidvania=>"Metroidvania",Self::PointAndClick=>"Point & Click",
            Self::SimCity=>"City Builder",Self::Custom(s)=>s,
        }
    }
    pub fn recommended_camera(&self) -> &str {
        match self {
            Self::Fps=>"first_person",Self::TopDown|Self::Strategy|Self::TowerDefense=>"top_down",
            Self::Platformer2D|Self::Metroidvania|Self::PointAndClick=>"side_scroll",
            Self::VisualNovel=>"static_portrait",Self::SpaceShooter=>"top_down_2d",
            _=>"third_person",
        }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum SetupDifficulty { Beginner, Intermediate, Advanced }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TemplateScene { pub name:String, pub path:String, pub start_scene:bool, pub description:String }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct TemplateScript { pub name:String, pub path:String, pub content:String, pub attach_to:Option<String> }
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AssetRef { pub id:String, pub kind:String, pub source:String }

pub struct TemplateRegistry {
    pub templates: HashMap<String,ProjectTemplate>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut r = Self { templates:HashMap::new() };
        r.register_defaults();
        r
    }

    fn register_defaults(&mut self) {
        let templates = vec![
            ProjectTemplate {
                id:"blank".to_string(), name:"Blank Project".to_string(),
                description:"Empty project — build anything from scratch.".to_string(),
                genre:Genre::Blank, preview_img:"previews/blank.png".to_string(),
                scenes:vec![TemplateScene{name:"main".to_string(),path:"scenes/main.scene".to_string(),start_scene:true,description:"Empty scene".to_string()}],
                scripts:vec![TemplateScript{name:"main".to_string(),path:"scripts/main.cv".to_string(),content:"@ready\nfn start():\n    pass\n".to_string(),attach_to:None}],
                assets:Vec::new(), features:vec!["Empty scene".to_string(),"Basic lighting".to_string()],
                difficulty:SetupDifficulty::Beginner, tags:vec!["empty".to_string(),"minimal".to_string()],
                ai_prompt:"Create a blank game project template".to_string(),
            },
            ProjectTemplate {
                id:"fps".to_string(), name:"First Person Shooter".to_string(),
                description:"FPS with player controller, weapon system, enemies, and health.".to_string(),
                genre:Genre::Fps, preview_img:"previews/fps.png".to_string(),
                scenes:vec![
                    TemplateScene{name:"main_menu".to_string(),path:"scenes/main_menu.scene".to_string(),start_scene:true,description:"Main menu".to_string()},
                    TemplateScene{name:"level_01".to_string(),path:"scenes/level_01.scene".to_string(),start_scene:false,description:"First level".to_string()},
                ],
                scripts:vec![
                    TemplateScript{name:"fps_controller".to_string(),path:"scripts/fps_controller.cv".to_string(),
                        content:"@ready\nfn start():\n    mouse_captured = true\n    health = 100\n\n@update\nfn tick(dt):\n    handle_movement(dt)\n    handle_shooting()\n\nfn handle_movement(dt):\n    var dir = input.get_move_dir()\n    self.move(dir * speed * dt)\n\nfn handle_shooting():\n    if input.just_pressed(\"fire\"):\n        weapon.shoot()\n".to_string(),attach_to:Some("Player".to_string())},
                    TemplateScript{name:"enemy_ai".to_string(),path:"scripts/enemy_ai.cv".to_string(),
                        content:"@ready\nfn start():\n    state = \"patrol\"\n    health = 50\n\n@update\nfn tick(dt):\n    match state:\n        \"patrol\": do_patrol(dt)\n        \"chase\":  chase_player(dt)\n        \"attack\": do_attack(dt)\n".to_string(),attach_to:Some("Enemy".to_string())},
                ],
                assets:Vec::new(),
                features:vec!["WASD+Mouse controller".to_string(),"Weapon system".to_string(),"Enemy AI".to_string(),"Health/Damage".to_string(),"HUD".to_string()],
                difficulty:SetupDifficulty::Intermediate, tags:vec!["fps","shooter","3d"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"FPS game template with player controller, weapons, enemies, and basic level".to_string(),
            },
            ProjectTemplate {
                id:"rpg".to_string(), name:"3D RPG".to_string(),
                description:"RPG with quests, NPC dialogue, inventory, stats, and world exploration.".to_string(),
                genre:Genre::Rpg, preview_img:"previews/rpg.png".to_string(),
                scenes:vec![
                    TemplateScene{name:"main_menu".to_string(),path:"scenes/main_menu.scene".to_string(),start_scene:true,description:"Main menu".to_string()},
                    TemplateScene{name:"world".to_string(),path:"scenes/world.scene".to_string(),start_scene:false,description:"Open world".to_string()},
                    TemplateScene{name:"village".to_string(),path:"scenes/village.scene".to_string(),start_scene:false,description:"Starting village".to_string()},
                ],
                scripts:vec![
                    TemplateScript{name:"player_rpg".to_string(),path:"scripts/player_rpg.cv".to_string(),
                        content:"@ready\nfn start():\n    stats = {hp:100, mp:50, str:10, def:5, lvl:1, xp:0}\n    inventory = []\n    quests = []\n\n@update\nfn tick(dt):\n    handle_movement(dt)\n    if input.just_pressed(\"interact\"):\n        try_interact()\n\nfn gain_xp(amount):\n    stats.xp += amount\n    if stats.xp >= xp_needed():\n        level_up()\n".to_string(),attach_to:Some("Player".to_string())},
                    TemplateScript{name:"npc_dialogue".to_string(),path:"scripts/npc_dialogue.cv".to_string(),
                        content:"@ready\nfn start():\n    dialogue_tree = load(dialogue_file)\n    ai_enabled = true\n\n@on(\"player_interact\")\nfn talk():\n    if ai_enabled:\n        var response = ai.chat(system: character_prompt, user: player.last_speech)\n        show_dialogue(response)\n    else:\n        show_dialogue(dialogue_tree.current)\n".to_string(),attach_to:Some("NPC".to_string())},
                ],
                assets:Vec::new(),
                features:vec!["Third-person controller".to_string(),"Stats system".to_string(),"Inventory".to_string(),"Quest tracker".to_string(),"AI dialogue".to_string(),"World map".to_string()],
                difficulty:SetupDifficulty::Advanced, tags:vec!["rpg","open-world","dialogue"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"RPG template with character stats, inventory, quest system, and NPC dialogue".to_string(),
            },
            ProjectTemplate {
                id:"platformer".to_string(), name:"3D Platformer".to_string(),
                description:"Platformer with tight jump controls, collectables, and level progression.".to_string(),
                genre:Genre::Platformer3D, preview_img:"previews/platformer.png".to_string(),
                scenes:vec![TemplateScene{name:"world_01".to_string(),path:"scenes/world_01.scene".to_string(),start_scene:true,description:"First world".to_string()}],
                scripts:vec![TemplateScript{name:"platformer_controller".to_string(),path:"scripts/platformer_controller.cv".to_string(),
                    content:"@ready\nfn start():\n    jumps_left = 2\n    coyote_time = 0.12\n    jump_buffer = 0.1\n\n@update\nfn tick(dt):\n    var on_floor = is_on_floor()\n    if on_floor:\n        jumps_left = 2\n        coyote_timer = coyote_time\n    else:\n        coyote_timer -= dt\n    if input.just_pressed(\"jump\"):\n        jump_buffer_timer = jump_buffer\n    jump_buffer_timer -= dt\n    if jump_buffer_timer > 0 and (on_floor or coyote_timer > 0 or jumps_left > 0):\n        do_jump()\n        jump_buffer_timer = 0\n        if not on_floor and coyote_timer <= 0:\n            jumps_left -= 1\n".to_string(),attach_to:Some("Player".to_string())}],
                assets:Vec::new(),
                features:vec!["Double jump + coyote time".to_string(),"Jump buffering".to_string(),"Collectables".to_string(),"Checkpoint system".to_string()],
                difficulty:SetupDifficulty::Intermediate, tags:vec!["platformer","3d","jump"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"3D platformer with responsive jump controls, coyote time, and level design".to_string(),
            },
            ProjectTemplate {
                id:"horror".to_string(), name:"Survival Horror".to_string(),
                description:"Atmospheric horror with dynamic AI enemies, limited resources, and tension mechanics.".to_string(),
                genre:Genre::Horror, preview_img:"previews/horror.png".to_string(),
                scenes:vec![TemplateScene{name:"mansion".to_string(),path:"scenes/mansion.scene".to_string(),start_scene:true,description:"Dark mansion".to_string()}],
                scripts:vec![TemplateScript{name:"horror_player".to_string(),path:"scripts/horror_player.cv".to_string(),
                    content:"@ready\nfn start():\n    sanity = 100.0\n    stamina = 100.0\n    items = []\n\n@update\nfn tick(dt):\n    update_sanity(dt)\n    update_stamina(dt)\n\nfn update_sanity(dt):\n    if near_monster():\n        sanity -= 10 * dt\n    elif in_light():\n        sanity += 5 * dt\n    sanity = clamp(sanity, 0, 100)\n    if sanity < 20:\n        apply_insanity_effects()\n".to_string(),attach_to:Some("Player".to_string())}],
                assets:Vec::new(),
                features:vec!["Sanity system".to_string(),"Dynamic AI monster".to_string(),"Flashlight".to_string(),"Inventory".to_string(),"Tension music".to_string()],
                difficulty:SetupDifficulty::Advanced, tags:vec!["horror","survival","atmospheric"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"Survival horror with sanity mechanics, limited resources, and atmospheric tension".to_string(),
            },
            ProjectTemplate {
                id:"open_world".to_string(), name:"Open World".to_string(),
                description:"Vast open world with procedural terrain, dynamic weather, day/night, and exploration.".to_string(),
                genre:Genre::OpenWorld, preview_img:"previews/open_world.png".to_string(),
                scenes:vec![TemplateScene{name:"world".to_string(),path:"scenes/world.scene".to_string(),start_scene:true,description:"Open world".to_string()}],
                scripts:vec![TemplateScript{name:"world_manager".to_string(),path:"scripts/world_manager.cv".to_string(),
                    content:"@ready\nfn start():\n    world_streaming.enabled = true\n    world_streaming.chunk_size = 256\n    weather.current = \"Clear\"\n    time.hour = 8.0\n\n@update\nfn tick(dt):\n    world_streaming.update(player.position, dt)\n    time.advance(dt)\n    if time.should_change_weather():\n        weather.transition_to(weather.pick_next())\n".to_string(),attach_to:Some("WorldManager".to_string())}],
                assets:Vec::new(),
                features:vec!["256m chunk streaming".to_string(),"Procedural terrain".to_string(),"30+ biomes".to_string(),"Dynamic weather".to_string(),"Day/night cycle".to_string(),"Creature ecosystem".to_string()],
                difficulty:SetupDifficulty::Advanced, tags:vec!["open-world","exploration","procedural"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"Open world with procedural generation, biomes, weather, and exploration".to_string(),
            },
            ProjectTemplate {
                id:"vn".to_string(), name:"Visual Novel".to_string(),
                description:"Visual novel with branching dialogue, character sprites, CG scenes, and multiple endings.".to_string(),
                genre:Genre::VisualNovel, preview_img:"previews/vn.png".to_string(),
                scenes:vec![TemplateScene{name:"chapter_01".to_string(),path:"scenes/chapter_01.scene".to_string(),start_scene:true,description:"First chapter".to_string()}],
                scripts:vec![TemplateScript{name:"vn_engine".to_string(),path:"scripts/vn_engine.cv".to_string(),
                    content:"@ready\nfn start():\n    script = load(\"scripts/story/chapter_01.script\")\n    choices_made = {}\n    relationship_flags = {}\n\nfn show_line(speaker, text, portrait):\n    ui.dialogue.show(speaker, text, portrait)\n    audio.voice.play(get_voice_file(speaker))\n    await input.any_key()\n\nfn show_choice(options):\n    var picked = await ui.choice_menu.show(options)\n    choices_made[current_node] = picked\n    if picked.has_flag:\n        relationship_flags[picked.flag] = true\n    return picked\n".to_string(),attach_to:None}],
                assets:Vec::new(),
                features:vec!["Branching dialogue".to_string(),"Sprite system".to_string(),"CG viewer".to_string(),"Save anywhere".to_string(),"Multiple endings".to_string(),"Voice acting".to_string()],
                difficulty:SetupDifficulty::Beginner, tags:vec!["visual-novel","story","dialogue"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"Visual novel engine with branching story, character sprites, and multiple endings".to_string(),
            },
            ProjectTemplate {
                id:"roguelike".to_string(), name:"Roguelike".to_string(),
                description:"Procedural dungeon roguelike with permadeath, run-based progression, and deep builds.".to_string(),
                genre:Genre::Roguelike, preview_img:"previews/roguelike.png".to_string(),
                scenes:vec![TemplateScene{name:"dungeon".to_string(),path:"scenes/dungeon.scene".to_string(),start_scene:true,description:"Procedural dungeon".to_string()}],
                scripts:vec![TemplateScript{name:"run_manager".to_string(),path:"scripts/run_manager.cv".to_string(),
                    content:"@ready\nfn start():\n    floor = 1\n    seed = rng.new_seed()\n    run_stats = {kills:0, gold:0, time:0}\n    generate_floor()\n\nfn generate_floor():\n    var gen = DungeonGenerator.new(seed, floor)\n    gen.rooms = 8 + floor * 2\n    gen.enemies_per_room = 2 + floor\n    gen.boss = floor % 5 == 0\n    gen.build()\n    floor += 1\n\nfn on_player_death():\n    save_run_record(run_stats)\n    show_death_screen()\n".to_string(),attach_to:Some("RunManager".to_string())}],
                assets:Vec::new(),
                features:vec!["Procedural dungeons".to_string(),"Permadeath".to_string(),"Run progression".to_string(),"Deep build system".to_string(),"Meta-progression".to_string()],
                difficulty:SetupDifficulty::Advanced, tags:vec!["roguelike","procedural","permadeath"].iter().map(|s|s.to_string()).collect(),
                ai_prompt:"Roguelike with procedural dungeon generation, permadeath, and run-based progression".to_string(),
            },
        ];

        for t in templates { self.templates.insert(t.id.clone(), t); }
        tracing::info!("Template registry: {} templates loaded", self.templates.len());
    }

    pub fn get(&self, id:&str) -> Option<&ProjectTemplate> { self.templates.get(id) }
    pub fn all(&self) -> Vec<&ProjectTemplate> { self.templates.values().collect() }
    pub fn by_genre(&self, genre:&str) -> Vec<&ProjectTemplate> {
        self.templates.values().filter(|t| t.genre.name().to_lowercase()==genre.to_lowercase()).collect()
    }
    pub fn count(&self) -> usize { self.templates.len() }
}

impl Default for TemplateRegistry { fn default() -> Self { Self::new() } }
extern crate tracing;
