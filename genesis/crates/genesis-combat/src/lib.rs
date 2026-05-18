//! Combat System — hitboxes, damage pipeline, status effects, abilities, combos
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use chrono::{DateTime,Utc};

// ═══ DAMAGE ═══════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DamageType {
    Physical, Blunt, Piercing, Slashing,
    Fire, Ice, Lightning, Poison, Acid, Necrotic, Radiant,
    Psychic, Force, Sonic, Arcane, Holy, Shadow,
    True,  // ignores all resistances
    Custom(String),
}

impl DamageType {
    pub fn element(&self) -> &str {
        match self {
            Self::Fire=>"fire", Self::Ice=>"ice", Self::Lightning=>"lightning",
            Self::Poison=>"poison", Self::Acid=>"acid", Self::Holy=>"holy",
            Self::Shadow=>"shadow", Self::Arcane=>"arcane", _=>"physical",
        }
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DamageEvent {
    pub id:           String,
    pub source:       String,      // entity_id
    pub target:       String,
    pub base_damage:  f32,
    pub final_damage: f32,
    pub kind:         DamageType,
    pub crit:         bool,
    pub crit_mult:    f32,
    pub blocked:      f32,
    pub resisted:     f32,
    pub absorbed:     f32,
    pub overkill:     f32,
    pub position:     [f32;3],
    pub hitbox_id:    Option<String>,
    pub ability_id:   Option<String>,
    pub flags:        Vec<DamageFlag>,
    pub ts:           DateTime<Utc>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum DamageFlag { Dot, Aoe, Reflected, Thorns, Lifesteal, Projectile, Melee, Ranged }

impl DamageEvent {
    pub fn new(src:&str, tgt:&str, base:f32, kind:DamageType, pos:[f32;3]) -> Self {
        Self { id:format!("dmg_{}", Utc::now().timestamp_millis()), source:src.to_string(),
               target:tgt.to_string(), base_damage:base, final_damage:base, kind,
               crit:false, crit_mult:1.5, blocked:0.0, resisted:0.0, absorbed:0.0,
               overkill:0.0, position:pos, hitbox_id:None, ability_id:None,
               flags:Vec::new(), ts:Utc::now() }
    }
    pub fn with_crit(mut self) -> Self { self.crit=true; self.final_damage*=self.crit_mult; self }
    pub fn with_flag(mut self, f:DamageFlag) -> Self { self.flags.push(f); self }
    pub fn net(&self) -> f32 { (self.final_damage - self.blocked - self.resisted - self.absorbed).max(0.0) }
}

// ═══ HITBOX ═══════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Hitbox {
    pub id:           String,
    pub entity_id:    String,
    pub kind:         HitboxKind,
    pub shape:        HitboxShape,
    pub offset:       [f32;3],
    pub rotation:     [f32;3],
    pub damage_mult:  f32,      // multiplier for damage dealt to this box
    pub crit_mult:    f32,      // additional crit mult
    pub active:       bool,
    pub active_frames:[u32;2],  // frame range when active (for attack hitboxes)
    pub tags:         Vec<String>,
    pub layer:        u32,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum HitboxKind { Hurtbox, Attackbox, Projectile, Sensor, Shield }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum HitboxShape {
    Sphere  { r:f32 },
    Box     { he:[f32;3] },
    Capsule { hh:f32, r:f32 },
    Cylinder{ hh:f32, r:f32 },
}

impl HitboxShape {
    pub fn volume(&self) -> f32 {
        match self {
            Self::Sphere{r}         => 4.0/3.0 * std::f32::consts::PI * r*r*r,
            Self::Box{he}           => 8.0*he[0]*he[1]*he[2],
            Self::Capsule{hh,r}     => std::f32::consts::PI*r*r*(2.0*hh + 4.0/3.0*r),
            Self::Cylinder{hh,r}    => std::f32::consts::PI*r*r*2.0*hh,
        }
    }
}

// ═══ STATUS EFFECTS ═══════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct StatusEffect {
    pub id:           String,
    pub kind:         StatusKind,
    pub duration:     f32,
    pub elapsed:      f32,
    pub tick_rate:    f32,
    pub tick_timer:   f32,
    pub stacks:       u32,
    pub max_stacks:   u32,
    pub source:       String,
    pub params:       HashMap<String,f32>,
    pub dispellable:  bool,
    pub icon:         String,
    pub color:        [f32;4],
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum StatusKind {
    // Offensive DoT
    Burning    { dps:f32 }, Poisoned { dps:f32, stacks:u32 },
    Bleeding   { dps:f32 }, Shocked  { dps:f32, stun_chance:f32 },
    Frozen     { slow:f32 }, Cursed   { stat_reduce:f32 },
    Corroding  { armor_reduce:f32 }, Withering { max_hp_reduce:f32 },
    // Control
    Stunned, Silenced, Blinded { miss_chance:f32 }, Feared { duration:f32 },
    Rooted, Slowed { mult:f32 }, Confused { chance:f32 },
    Charmed { src:String }, Petrified, Sleeping { break_on_dmg:bool },
    Taunted { target:String },
    // Defensive / Positive
    Shielded   { amount:f32 }, Regenerating { hps:f32 },
    Hasted     { speed_mult:f32 }, Invisible,
    Invincible { duration:f32 }, Blessed { stat_bonus:f32 },
    Buffed     { stat:String, bonus:f32 },
    // Utility
    Marked     { radius:f32 }, Exposed { dmg_taken_mult:f32 },
    Weakened   { dmg_mult:f32 }, Empowered { dmg_mult:f32 },
    Custom     { name:String, desc:String },
}

impl StatusEffect {
    pub fn burning(src:&str, dps:f32, dur:f32) -> Self {
        Self { id:format!("se_{}", Utc::now().timestamp_millis()), kind:StatusKind::Burning{dps},
               duration:dur, elapsed:0.0, tick_rate:1.0, tick_timer:0.0, stacks:1, max_stacks:3,
               source:src.to_string(), params:HashMap::new(), dispellable:true,
               icon:"status/burn.png".to_string(), color:[1.0,0.4,0.0,1.0] }
    }
    pub fn is_expired(&self) -> bool { self.elapsed >= self.duration }
    pub fn remaining(&self) -> f32 { (self.duration-self.elapsed).max(0.0) }
    pub fn pct(&self) -> f32 { (self.elapsed/self.duration).clamp(0.0,1.0) }
    pub fn tick(&mut self, delta:f32) -> bool {
        self.elapsed += delta; self.tick_timer += delta;
        if self.tick_timer >= self.tick_rate { self.tick_timer=0.0; return true; }
        false
    }
    pub fn is_cc(&self) -> bool {
        matches!(self.kind, StatusKind::Stunned|StatusKind::Rooted|StatusKind::Frozen{..}|StatusKind::Sleeping{..}|StatusKind::Petrified|StatusKind::Confused{..})
    }
}

// ═══ ABILITIES ════════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Ability {
    pub id:           String,
    pub name:         String,
    pub description:  String,
    pub icon:         String,
    pub kind:         AbilityKind,
    pub cost:         AbilityCost,
    pub cooldown:     f32,
    pub cast_time:    f32,         // 0 = instant
    pub range:        f32,
    pub radius:       f32,
    pub damage:       Option<DamageTemplate>,
    pub effects:      Vec<String>,
    pub animation:    String,
    pub vfx:          String,
    pub sfx:          String,
    pub interruptible:bool,
    pub charges:      u32,
    pub max_charges:  u32,
    pub charge_regen: f32,
    pub tags:         Vec<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum AbilityKind {
    Active, Passive, Toggle, Channeled{tick_rate:f32},
    Charged{min_time:f32,max_time:f32},
    Combo{sequence:Vec<String>},
    Reaction{trigger:String},
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct AbilityCost { pub health:f32, pub mana:f32, pub stamina:f32, pub charges:u32 }
impl Default for AbilityCost { fn default() -> Self { Self { health:0.0, mana:0.0, stamina:0.0, charges:0 } } }

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct DamageTemplate {
    pub base:  f32, pub scaling:Vec<(String,f32)>,  // (stat_name, coefficient)
    pub kind:  DamageType, pub crit_chance:f32, pub crit_mult:f32,
    pub spread:f32,  // damage variance ± pct
}

// ═══ COMBO SYSTEM ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct ComboState {
    pub inputs:      Vec<String>,       // sequence of ability/action IDs
    pub window:      f32,               // time allowed between inputs
    pub timer:       f32,
    pub active:      bool,
    pub combo_count: u32,
    pub multiplier:  f32,
    pub last_input:  Option<DateTime<Utc>>,
}

impl ComboState {
    pub fn new(window:f32) -> Self { Self { inputs:Vec::new(), window, timer:0.0, active:false, combo_count:0, multiplier:1.0, last_input:None } }
    pub fn push(&mut self, input:&str) {
        self.inputs.push(input.to_string());
        self.timer = self.window;
        self.combo_count += 1;
        self.multiplier = (1.0 + self.combo_count as f32 * 0.1).min(3.0);
        self.last_input = Some(Utc::now());
    }
    pub fn reset(&mut self) { self.inputs.clear(); self.timer=0.0; self.combo_count=0; self.multiplier=1.0; }
    pub fn tick(&mut self, delta:f32) {
        if self.timer > 0.0 { self.timer -= delta; if self.timer <= 0.0 { self.reset(); } }
    }
}

// ═══ COMBAT STATS ════════════════════════════════════════════════
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CombatStats {
    pub health:       f32, pub max_health:f32,
    pub mana:         f32, pub max_mana:f32,
    pub stamina:      f32, pub max_stamina:f32,
    pub shield:       f32, pub max_shield:f32,
    pub armor:        f32,          // flat damage reduction
    pub armor_pct:    f32,          // percentage damage reduction
    pub resistances:  HashMap<String,f32>,  // element → 0-1 resistance
    pub attack_power: f32,
    pub ability_power:f32,
    pub crit_chance:  f32,
    pub crit_mult:    f32,
    pub attack_speed: f32,
    pub move_speed:   f32,
    pub lifesteal:    f32,
    pub tenacity:     f32,          // CC duration reduction 0-1
    pub haste:        f32,          // CDR 0-1
}

impl CombatStats {
    pub fn player() -> Self {
        let mut resistances = HashMap::new();
        for el in &["fire","ice","lightning","poison"] { resistances.insert(el.to_string(),0.0); }
        Self { health:100.0, max_health:100.0, mana:100.0, max_mana:100.0,
               stamina:100.0, max_stamina:100.0, shield:0.0, max_shield:0.0,
               armor:5.0, armor_pct:0.0, resistances, attack_power:10.0,
               ability_power:10.0, crit_chance:0.05, crit_mult:1.5,
               attack_speed:1.0, move_speed:5.0, lifesteal:0.0, tenacity:0.0, haste:0.0 }
    }
    pub fn health_pct(&self) -> f32 { (self.health/self.max_health).clamp(0.0,1.0) }
    pub fn mana_pct(&self) -> f32 { (self.mana/self.max_mana).clamp(0.0,1.0) }
    pub fn stamina_pct(&self) -> f32 { (self.stamina/self.max_stamina).clamp(0.0,1.0) }
    pub fn is_alive(&self) -> bool { self.health > 0.0 }
    pub fn is_low_hp(&self) -> bool { self.health_pct() < 0.25 }
    pub fn resistance(&self, element:&str) -> f32 { *self.resistances.get(element).unwrap_or(&0.0) }
    pub fn regen(&mut self, delta:f32) {
        self.stamina = (self.stamina + 10.0*delta).min(self.max_stamina);
        self.mana    = (self.mana    +  5.0*delta).min(self.max_mana);
    }
}

// ═══ COMBAT MANAGER ══════════════════════════════════════════════
pub struct CombatManager {
    pub abilities:       HashMap<String,Ability>,
    pub hitboxes:        HashMap<String,Vec<Hitbox>>,      // entity_id → hitboxes
    pub status_effects:  HashMap<String,Vec<StatusEffect>>, // entity_id → effects
    pub combat_stats:    HashMap<String,CombatStats>,
    pub combos:          HashMap<String,ComboState>,
    pub damage_log:      std::collections::VecDeque<DamageEvent>,
    pub log_capacity:    usize,
    pub friendly_fire:   bool,
    pub team_map:        HashMap<String,u32>,  // entity_id → team
    pub total_damage:    f64,
    pub total_kills:     u64,
    pub total_crits:     u64,
}

impl CombatManager {
    pub fn new() -> Self {
        Self { abilities:HashMap::new(), hitboxes:HashMap::new(), status_effects:HashMap::new(),
               combat_stats:HashMap::new(), combos:HashMap::new(),
               damage_log:std::collections::VecDeque::new(), log_capacity:500,
               friendly_fire:false, team_map:HashMap::new(), total_damage:0.0,
               total_kills:0, total_crits:0 }
    }

    pub fn register_ability(&mut self, a:Ability) { self.abilities.insert(a.id.clone(), a); }

    pub fn register_entity(&mut self, entity_id:&str, stats:CombatStats, team:u32) {
        self.combat_stats.insert(entity_id.to_string(), stats);
        self.team_map.insert(entity_id.to_string(), team);
        self.status_effects.insert(entity_id.to_string(), Vec::new());
        self.hitboxes.insert(entity_id.to_string(), Vec::new());
        self.combos.insert(entity_id.to_string(), ComboState::new(0.5));
    }

    pub fn process_damage(&mut self, mut ev:DamageEvent) -> f32 {
        let tgt = ev.target.clone();
        let Some(stats) = self.combat_stats.get_mut(&tgt) else { return 0.0 };
        // Apply resistances
        let res = stats.resistance(ev.kind.element());
        ev.resisted = ev.final_damage * res;
        // Apply armor
        let after_res = ev.final_damage - ev.resisted;
        let after_armor = (after_res - stats.armor).max(after_res * (1.0-stats.armor_pct));
        // Apply shield
        let after_shield = if stats.shield > 0.0 {
            ev.absorbed = after_armor.min(stats.shield);
            stats.shield = (stats.shield - ev.absorbed).max(0.0);
            after_armor - ev.absorbed
        } else { after_armor };
        ev.final_damage = after_shield.max(0.0);
        // Apply to health
        let prev_hp = stats.health;
        stats.health = (stats.health - ev.final_damage).max(0.0);
        if prev_hp > 0.0 && stats.health <= 0.0 {
            ev.overkill = ev.final_damage - prev_hp;
            self.total_kills += 1;
            tracing::info!("Entity killed: {} by {}", tgt, ev.source);
        }
        if ev.crit { self.total_crits += 1; }
        self.total_damage += ev.final_damage as f64;
        let net = ev.final_damage;
        self.damage_log.push_back(ev);
        while self.damage_log.len() > self.log_capacity { self.damage_log.pop_front(); }
        net
    }

    pub fn apply_status(&mut self, entity_id:&str, mut effect:StatusEffect) {
        let effects = self.status_effects.entry(entity_id.to_string()).or_default();
        // Check if stackable
        if let Some(existing) = effects.iter_mut().find(|e| std::mem::discriminant(&e.kind) == std::mem::discriminant(&effect.kind)) {
            existing.stacks = (existing.stacks + 1).min(existing.max_stacks);
            existing.elapsed = 0.0;  // refresh duration
        } else {
            tracing::debug!("Applied status to {}: {:?}", entity_id, effect.kind);
            effects.push(effect);
        }
    }

    pub fn remove_status(&mut self, entity_id:&str, kind_name:&str) {
        if let Some(effects) = self.status_effects.get_mut(entity_id) {
            effects.retain(|e| format!("{:?}", e.kind).to_lowercase() != kind_name.to_lowercase());
        }
    }

    pub fn has_status(&self, entity_id:&str, kind_name:&str) -> bool {
        self.status_effects.get(entity_id).map(|effects|
            effects.iter().any(|e| format!("{:?}", e.kind).to_lowercase().contains(kind_name))
        ).unwrap_or(false)
    }

    pub fn is_cc(&self, entity_id:&str) -> bool {
        self.status_effects.get(entity_id).map(|effects| effects.iter().any(|e|e.is_cc())).unwrap_or(false)
    }

    pub fn tick(&mut self, delta:f32) {
        // Tick status effects
        for (entity_id, effects) in &mut self.status_effects {
            for effect in effects.iter_mut() {
                let ticked = effect.tick(delta);
                if ticked {
                    // Apply DoT damage
                    if let StatusKind::Burning{dps} = &effect.kind.clone() {
                        let dmg = DamageEvent::new(&effect.source.clone(), entity_id, dps*effect.stacks as f32, DamageType::Fire, [0.0;3]);
                        if let Some(stats) = self.combat_stats.get_mut(entity_id) {
                            stats.health = (stats.health - dmg.net()).max(0.0);
                        }
                    }
                    if let StatusKind::Poisoned{dps,..} = &effect.kind.clone() {
                        if let Some(stats) = self.combat_stats.get_mut(entity_id) {
                            stats.health = (stats.health - dps).max(0.0);
                        }
                    }
                }
            }
            // Remove expired effects
            effects.retain(|e| !e.is_expired());
        }
        // Tick stat regen
        for stats in self.combat_stats.values_mut() { stats.regen(delta); }
        // Tick combos
        for combo in self.combos.values_mut() { combo.tick(delta); }
    }

    pub fn is_alive(&self, entity_id:&str) -> bool {
        self.combat_stats.get(entity_id).map(|s|s.is_alive()).unwrap_or(false)
    }
    pub fn get_stats(&self, id:&str) -> Option<&CombatStats> { self.combat_stats.get(id) }
    pub fn get_stats_mut(&mut self, id:&str) -> Option<&mut CombatStats> { self.combat_stats.get_mut(id) }
    pub fn ability_count(&self) -> usize { self.abilities.len() }
    pub fn entity_count(&self) -> usize { self.combat_stats.len() }
    pub fn crit_rate(&self) -> f32 { if self.total_damage>0.0 { self.total_crits as f32/self.damage_log.len().max(1) as f32 } else { 0.0 } }
}

impl Default for CombatManager { fn default() -> Self { Self::new() } }
extern crate tracing;
