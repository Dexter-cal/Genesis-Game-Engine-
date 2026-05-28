//! Interior Design Agent
//!
//! Autonomously designs interiors for any space:
//! - Furniture placement (follows design principles)
//! - Lighting design (ambient, accent, task)
//! - Material/texture selection per style
//! - Color palette application
//! - Prop dressing (books, plants, clutter, art)
//! - Style matching (modern, medieval, sci-fi, etc.)
//! - Space planning (traffic flow, focal points)
//! - Procedural room generation from description
//! - Can CREATE new furniture meshes via GPU Lab
//! - Learns from reference images (vision model)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::info;
use genesis_core::events::{AgentEvent, EventEnvelope};
use crate::base::{Agent, AgentBase, AgentContext, AgentSoul, AgentStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteriorStyle {
    Modern, Minimalist, Industrial, Scandinavian,
    Bohemian, Rustic, FarmHouse, MidCentury,
    // Game-specific
    Medieval, Fantasy, Viking, Japanese, Egyptian,
    SciFi, Cyberpunk, SpaceStation, PostApocalyptic,
    Victorian, Baroque, Art_Deco, Steampunk,
    Dungeon, Tavern, CastleHall, Temple,
    Laboratory, Hospital, Prison, Abandoned,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDefinition {
    pub id: String,
    pub name: String,
    pub room_type: RoomType,
    pub style: InteriorStyle,
    pub floor_bounds: [f32; 4],   // x_min, z_min, x_max, z_max
    pub ceiling_height: f32,
    pub entry_points: Vec<[f32; 3]>,
    pub window_positions: Vec<WindowDef>,
    pub existing_props: Vec<String>,
    pub mood: RoomMood,
    pub budget_quality: f32,  // 0-1 (0=rough, 1=luxury)
    pub occupied_by: Vec<String>, // NPC IDs who live/work here
    pub custom_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Bedroom, LivingRoom, Kitchen, Bathroom, Dining,
    Office, Library, Throne, Prison, Dungeon,
    Tavern, Bedroom_Inn, Shop, Workshop, Laboratory,
    Armory, Treasury, Chapel, Crypt, Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomMood {
    Cozy, Grand, Mysterious, Menacing, Peaceful,
    Chaotic, Clinical, Sacred, Abandoned, Festive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowDef {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurniturePlacement {
    pub furniture_id: String,
    pub asset_id: String,
    pub position: [f32; 3],
    pub rotation_y: f32,
    pub scale: f32,
    pub layer: FurnitureLayer,
    pub reason: String, // why it was placed here (design rationale)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FurnitureLayer {
    Architectural, // walls, floors, ceiling
    Large,         // sofas, beds, tables
    Medium,        // chairs, shelves, desks
    Small,         // lamps, plants, vases
    Accent,        // books, candles, art
    Decals,        // dirt, scratches, stains
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPlan {
    pub room_id: String,
    pub style: InteriorStyle,
    pub placements: Vec<FurniturePlacement>,
    pub light_sources: Vec<LightDesign>,
    pub material_assignments: HashMap<String, String>, // surface_id → material_id
    pub color_palette: Vec<[f32; 4]>,
    pub narrative_notes: String, // story-driven design notes
    pub generated_assets: Vec<String>, // new assets created for this room
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightDesign {
    pub position: [f32; 3],
    pub light_type: String, // "point", "spot", "directional"
    pub color: [f32; 4],
    pub intensity: f32,
    pub purpose: String,    // "ambient", "accent", "task", "dramatic"
    pub asset_id: Option<String>, // lamp/fixture mesh
}

pub struct InteriorDesignAgent {
    base: AgentBase,
    design_plans: HashMap<String, DesignPlan>,
    furniture_library: HashMap<InteriorStyleKey, Vec<String>>, // style → asset IDs
    pending_rooms: Vec<RoomDefinition>,
}

type InteriorStyleKey = String;

impl InteriorDesignAgent {
    pub fn new() -> Self {
        Self {
            base: AgentBase::new(AgentSoul {
                id: "interior_design".to_string(),
                name: "Interior Design Agent".to_string(),
                description: "Autonomously designs interiors: furniture placement, lighting, materials, props. Can generate new assets and create custom furniture.".to_string(),
                system_prompt: r#"You are the Interior Design Agent for ChronoVerse.
You design believable, atmospheric, story-rich interiors for game spaces.

Your design principles:
1. STORY FIRST: Every prop tells a story about the inhabitant
2. FOCAL POINTS: Every room needs one strong focal point
3. TRAFFIC FLOW: Leave clear paths between entry points
4. LIGHTING LAYERS: Ambient + accent + task lights
5. RULE OF THIRDS: Asymmetric placement feels natural
6. STYLE CONSISTENCY: Every prop fits the established style
7. WEAR AND TEAR: Add age, damage, and personal touches

When given a room, output a JSON design plan with:
- Furniture placements with positions and rationale
- Light sources with purpose
- Material overrides
- Narrative notes explaining the inhabitant's story

You can request new assets from the GPU Lab agent if needed.
You can create new mechanics (like drawers opening, secret passages) using the Mechanics system."#.to_string(),
                tools: vec![
                    "place_entity".to_string(),
                    "set_material".to_string(),
                    "set_light".to_string(),
                    "query_asset_library".to_string(),
                    "request_new_asset".to_string(),
                    "create_mechanic".to_string(),
                    "apply_decal".to_string(),
                    "set_room_atmosphere".to_string(),
                    "analyze_reference_image".to_string(),
                    "create_tool".to_string(),
                ],
                listens_to: vec!["interior_design_requested".to_string()],
                can_emit: vec![
                    "interior_design_complete".to_string(),
                    "asset_3d_requested".to_string(),
                ],
                max_tokens_per_call: 3000,
                parallelizable: true,
                priority: 5,
            }),
            design_plans: HashMap::new(),
            furniture_library: HashMap::new(),
            pending_rooms: Vec::new(),
        }
    }

    async fn design_room(&mut self, room: RoomDefinition, ctx: &AgentContext) -> Result<DesignPlan> {
        self.base.set_working("Designing room layout", 0.0);
        info!("Interior Design: designing {} ({:?} style)", room.name, room.style);

        let style_key = format!("{:?}", room.style);
        let placements = self.plan_furniture(&room);
        let lights = self.plan_lighting(&room);
        let palette = self.select_palette(&room.style, &room.mood);

        let plan = DesignPlan {
            room_id: room.id.clone(),
            style: room.style.clone(),
            placements,
            light_sources: lights,
            material_assignments: HashMap::new(),
            color_palette: palette,
            narrative_notes: self.generate_narrative_notes(&room),
            generated_assets: Vec::new(),
        };

        // Emit completion
        ctx.emit(
            AgentEvent::AnalyticsEvent {
                event_type: "interior_design_complete".to_string(),
                data: serde_json::json!({ "room_id": room.id }),
            },
            self.id(),
        );

        self.base.set_idle();
        Ok(plan)
    }

    fn plan_furniture(&self, room: &RoomDefinition) -> Vec<FurniturePlacement> {
        let mut placements = Vec::new();
        let cx = (room.floor_bounds[0] + room.floor_bounds[2]) * 0.5;
        let cz = (room.floor_bounds[1] + room.floor_bounds[3]) * 0.5;

        // Focal point furniture (center or against main wall)
        match &room.room_type {
            RoomType::Bedroom => {
                placements.push(FurniturePlacement {
                    furniture_id: "bed_main".to_string(),
                    asset_id: format!("{:?}_bed", room.style).to_lowercase(),
                    position: [cx, 0.0, room.floor_bounds[3] - 1.5],
                    rotation_y: 180.0,
                    scale: 1.0,
                    layer: FurnitureLayer::Large,
                    reason: "Bed placed against far wall from entry for best traffic flow".to_string(),
                });
                placements.push(FurniturePlacement {
                    furniture_id: "nightstand_l".to_string(),
                    asset_id: "nightstand_wood".to_string(),
                    position: [cx - 0.9, 0.0, room.floor_bounds[3] - 1.5],
                    rotation_y: 0.0, scale: 1.0,
                    layer: FurnitureLayer::Medium,
                    reason: "Left nightstand flanking bed".to_string(),
                });
                placements.push(FurniturePlacement {
                    furniture_id: "nightstand_r".to_string(),
                    asset_id: "nightstand_wood".to_string(),
                    position: [cx + 0.9, 0.0, room.floor_bounds[3] - 1.5],
                    rotation_y: 0.0, scale: 1.0,
                    layer: FurnitureLayer::Medium,
                    reason: "Right nightstand flanking bed".to_string(),
                });
            }
            RoomType::Tavern => {
                // Scatter tables across floor
                let cols = 3;
                let rows = 2;
                for r in 0..rows {
                    for c in 0..cols {
                        placements.push(FurniturePlacement {
                            furniture_id: format!("table_{r}_{c}"),
                            asset_id: "tavern_table_round".to_string(),
                            position: [
                                room.floor_bounds[0] + 1.5 + c as f32 * 2.5,
                                0.0,
                                room.floor_bounds[1] + 1.5 + r as f32 * 2.5,
                            ],
                            rotation_y: (c * 45) as f32,
                            scale: 1.0,
                            layer: FurnitureLayer::Large,
                            reason: "Patron table for social gameplay".to_string(),
                        });
                    }
                }
            }
            _ => {
                // Generic center table
                placements.push(FurniturePlacement {
                    furniture_id: "center_piece".to_string(),
                    asset_id: "table_generic".to_string(),
                    position: [cx, 0.0, cz],
                    rotation_y: 0.0, scale: 1.0,
                    layer: FurnitureLayer::Large,
                    reason: "Central focal point".to_string(),
                });
            }
        }

        // Add style-specific accent props
        match &room.style {
            InteriorStyle::Medieval | InteriorStyle::Dungeon => {
                placements.push(FurniturePlacement {
                    furniture_id: "torch_wall_1".to_string(),
                    asset_id: "torch_wall_iron".to_string(),
                    position: [room.floor_bounds[0] + 0.1, 1.8, cz],
                    rotation_y: 90.0, scale: 1.0,
                    layer: FurnitureLayer::Accent,
                    reason: "Wall torch for medieval atmosphere".to_string(),
                });
            }
            InteriorStyle::SciFi | InteriorStyle::Cyberpunk => {
                placements.push(FurniturePlacement {
                    furniture_id: "hologram_display".to_string(),
                    asset_id: "holographic_terminal".to_string(),
                    position: [cx, 0.0, room.floor_bounds[1] + 1.0],
                    rotation_y: 0.0, scale: 1.0,
                    layer: FurnitureLayer::Medium,
                    reason: "Tech focal point for sci-fi setting".to_string(),
                });
            }
            _ => {}
        }

        placements
    }

    fn plan_lighting(&self, room: &RoomDefinition) -> Vec<LightDesign> {
        let mut lights = Vec::new();
        let cx = (room.floor_bounds[0] + room.floor_bounds[2]) * 0.5;
        let cz = (room.floor_bounds[1] + room.floor_bounds[3]) * 0.5;
        let ch = room.ceiling_height;

        // Ambient ceiling light
        let (ambient_color, ambient_intensity) = match &room.mood {
            RoomMood::Cozy       => ([1.0, 0.85, 0.65, 1.0], 1.5),
            RoomMood::Mysterious => ([0.4, 0.35, 0.7, 1.0], 0.6),
            RoomMood::Menacing   => ([0.7, 0.15, 0.1, 1.0], 0.5),
            RoomMood::Sacred     => ([1.0, 0.95, 0.75, 1.0], 2.5),
            RoomMood::Clinical   => ([0.9, 0.95, 1.0, 1.0], 3.0),
            _                    => ([1.0, 0.95, 0.85, 1.0], 2.0),
        };

        lights.push(LightDesign {
            position: [cx, ch - 0.2, cz],
            light_type: "point".to_string(),
            color: ambient_color,
            intensity: ambient_intensity,
            purpose: "ambient".to_string(),
            asset_id: self.ceiling_fixture_for_style(&room.style),
        });

        // Accent lights for windows
        for window in &room.window_positions {
            lights.push(LightDesign {
                position: [window.position[0], window.position[1], window.position[2]],
                light_type: "spot".to_string(),
                color: [1.0, 0.97, 0.9, 1.0],
                intensity: 1.2,
                purpose: "natural_daylight".to_string(),
                asset_id: None,
            });
        }

        lights
    }

    fn ceiling_fixture_for_style(&self, style: &InteriorStyle) -> Option<String> {
        Some(match style {
            InteriorStyle::Medieval | InteriorStyle::Dungeon => "chandelier_iron_candle",
            InteriorStyle::Fantasy => "magic_orb_ceiling",
            InteriorStyle::SciFi | InteriorStyle::Cyberpunk => "led_panel_ceiling",
            InteriorStyle::Modern | InteriorStyle::Minimalist => "recessed_light",
            InteriorStyle::Victorian | InteriorStyle::Baroque => "chandelier_crystal",
            InteriorStyle::Tavern => "lantern_ceiling_wood",
            _ => "ceiling_light_generic",
        }.to_string())
    }

    fn select_palette(&self, style: &InteriorStyle, mood: &RoomMood) -> Vec<[f32; 4]> {
        match style {
            InteriorStyle::Medieval => vec![
                [0.35, 0.25, 0.15, 1.0], // dark wood
                [0.6, 0.55, 0.45, 1.0],  // stone grey
                [0.7, 0.5, 0.2, 1.0],    // warm torchlight
                [0.15, 0.12, 0.1, 1.0],  // deep shadow
            ],
            InteriorStyle::Cyberpunk => vec![
                [0.05, 0.05, 0.08, 1.0], // near black
                [0.0, 0.8, 1.0, 1.0],    // neon cyan
                [1.0, 0.1, 0.5, 1.0],    // neon pink
                [0.1, 0.15, 0.2, 1.0],   // dark metal
            ],
            InteriorStyle::SciFi => vec![
                [0.1, 0.12, 0.15, 1.0],
                [0.0, 0.6, 0.9, 1.0],
                [0.9, 0.95, 1.0, 1.0],
                [0.2, 0.5, 0.7, 1.0],
            ],
            _ => vec![
                [0.85, 0.8, 0.72, 1.0],
                [0.4, 0.35, 0.3, 1.0],
                [0.7, 0.62, 0.5, 1.0],
                [0.95, 0.92, 0.88, 1.0],
            ],
        }
    }

    fn generate_narrative_notes(&self, room: &RoomDefinition) -> String {
        let owner = room.occupied_by.first().map(|s| s.as_str()).unwrap_or("an unknown occupant");
        format!(
            "This {} tells the story of {}. The {} style suggests {}. \
             Design goal: the player should immediately understand who lives here \
             without reading any text.",
            format!("{:?}", room.room_type).to_lowercase(),
            owner,
            format!("{:?}", room.style),
            match &room.mood {
                RoomMood::Cozy => "warmth and comfort, a safe haven",
                RoomMood::Mysterious => "secrets and hidden knowledge",
                RoomMood::Menacing => "danger and dark intent",
                _ => "the character's daily life",
            }
        )
    }
}

#[async_trait]
impl Agent for InteriorDesignAgent {
    fn id(&self) -> &str { "interior_design" }
    fn name(&self) -> &str { "Interior Design Agent" }
    fn description(&self) -> &str { "Autonomous interior design: furniture, lighting, materials, props" }
    fn status(&self) -> &AgentStatus { &self.base.status }
    fn subscriptions(&self) -> Vec<&'static str> { vec!["interior_design_requested"] }

    async fn handle_event(&mut self, event: &EventEnvelope<AgentEvent>, ctx: &AgentContext) -> Result<()> {
        // Listen for analytics events repurposed as interior requests
        if let AgentEvent::AnalyticsEvent { event_type, data } = &event.event {
            if event_type == "interior_design_requested" {
                if let Some(room_json) = data.get("room") {
                    if let Ok(room) = serde_json::from_value::<RoomDefinition>(room_json.clone()) {
                        let plan = self.design_room(room, ctx).await?;
                        info!("Interior design complete: {} placements", plan.placements.len());
                        self.design_plans.insert(plan.room_id.clone(), plan);
                    }
                }
            }
        }
        Ok(())
    }

    fn tick_interval(&self) -> Option<f32> { Some(1.0) }
    async fn tick(&mut self, ctx: &AgentContext, _delta: f32) -> Result<()> {
        if let Some(room) = self.pending_rooms.pop() {
            let plan = self.design_room(room, ctx).await?;
            self.design_plans.insert(plan.room_id.clone(), plan);
        }
        Ok(())
    }
}
