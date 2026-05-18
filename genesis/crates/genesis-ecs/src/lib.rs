//! ECS — Entity Component System (sparse-set, cache-friendly, parallel queries)
use std::collections::{HashMap, HashSet};
use std::any::{Any, TypeId};

// ═══ ENTITY ═══════════════════════════════════════════════════════
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id:         u32,
    pub generation: u32,
}
impl Entity {
    pub fn new(id:u32, gen:u32) -> Self { Self { id, generation:gen } }
    pub fn is_valid(&self) -> bool { self.id != u32::MAX }
    pub fn null() -> Self { Self { id:u32::MAX, generation:0 } }
}
impl std::fmt::Display for Entity {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "Entity({}:{})", self.id, self.generation) }
}

// ═══ COMPONENT STORAGE (sparse set) ══════════════════════════════
pub trait Component: Any + Send + Sync + 'static {}
impl<T: Any + Send + Sync + 'static> Component for T {}

struct ComponentVec {
    data:    Vec<Box<dyn Any + Send + Sync>>,
    sparse:  Vec<Option<usize>>,
    dense:   Vec<u32>,          // entity ids for each dense slot
}

impl ComponentVec {
    fn new() -> Self { Self { data:Vec::new(), sparse:Vec::new(), dense:Vec::new() } }

    fn insert(&mut self, entity_id:u32, component:Box<dyn Any+Send+Sync>) {
        let id = entity_id as usize;
        while self.sparse.len() <= id { self.sparse.push(None); }
        if let Some(idx) = self.sparse[id] {
            self.data[idx] = component;
        } else {
            let idx = self.data.len();
            self.sparse[id] = Some(idx);
            self.dense.push(entity_id);
            self.data.push(component);
        }
    }

    fn remove(&mut self, entity_id:u32) -> bool {
        let id = entity_id as usize;
        if id >= self.sparse.len() { return false; }
        if let Some(idx) = self.sparse[id] {
            let last_idx = self.data.len() - 1;
            if idx != last_idx {
                self.data.swap(idx, last_idx);
                let moved_entity = self.dense[last_idx];
                self.dense[idx] = moved_entity;
                self.sparse[moved_entity as usize] = Some(idx);
            }
            self.data.pop();
            self.dense.pop();
            self.sparse[id] = None;
            true
        } else { false }
    }

    fn get(&self, entity_id:u32) -> Option<&(dyn Any+Send+Sync)> {
        let id = entity_id as usize;
        if id >= self.sparse.len() { return None; }
        self.sparse[id].map(|idx| self.data[idx].as_ref())
    }

    fn get_mut(&mut self, entity_id:u32) -> Option<&mut (dyn Any+Send+Sync)> {
        let id = entity_id as usize;
        if id >= self.sparse.len() { return None; }
        self.sparse[id].map(|idx| self.data[idx].as_mut())
    }

    fn has(&self, entity_id:u32) -> bool {
        let id = entity_id as usize;
        id < self.sparse.len() && self.sparse[id].is_some()
    }

    fn entity_ids(&self) -> &[u32] { &self.dense }
    fn len(&self) -> usize { self.data.len() }
}

// ═══ BUILT-IN COMPONENTS ══════════════════════════════════════════
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: [f32;3],
    pub rotation: [f32;4],  // quaternion
    pub scale:    [f32;3],
}
impl Transform {
    pub fn identity() -> Self { Self { position:[0.0;3], rotation:[0.0,0.0,0.0,1.0], scale:[1.0;3] } }
    pub fn at(x:f32,y:f32,z:f32) -> Self { let mut t=Self::identity(); t.position=[x,y,z]; t }
}

#[derive(Debug, Clone)]
pub struct Name(pub String);
impl Name { pub fn new(s:&str) -> Self { Self(s.to_string()) } }

#[derive(Debug, Clone, Default)]
pub struct Tags(pub Vec<String>);
impl Tags {
    pub fn has(&self,t:&str)->bool { self.0.iter().any(|s|s==t) }
    pub fn add(&mut self,t:&str)   { if!self.has(t){self.0.push(t.to_string());} }
    pub fn remove(&mut self,t:&str){ self.0.retain(|s|s!=t); }
}

#[derive(Debug, Clone)]
pub struct Visibility(pub bool);
impl Visibility { pub fn visible() -> Self { Self(true) } pub fn hidden() -> Self { Self(false) } }

#[derive(Debug, Clone)]
pub struct Parent(pub Entity);

#[derive(Debug, Clone, Default)]
pub struct Children(pub Vec<Entity>);

#[derive(Debug, Clone)]
pub struct Health { pub current:f32, pub max:f32, pub regen:f32 }
impl Health {
    pub fn new(max:f32) -> Self { Self { current:max, max, regen:0.0 } }
    pub fn is_alive(&self) -> bool { self.current > 0.0 }
    pub fn pct(&self) -> f32 { (self.current/self.max).clamp(0.0,1.0) }
    pub fn damage(&mut self, amount:f32) { self.current = (self.current-amount).max(0.0); }
    pub fn heal(&mut self, amount:f32)   { self.current = (self.current+amount).min(self.max); }
    pub fn tick(&mut self, dt:f32)       { if self.regen>0.0 { self.heal(self.regen*dt); } }
}

#[derive(Debug, Clone)]
pub struct Velocity { pub linear:[f32;3], pub angular:[f32;3] }
impl Default for Velocity { fn default() -> Self { Self { linear:[0.0;3], angular:[0.0;3] } } }

#[derive(Debug, Clone)]
pub struct Script { pub path:String, pub enabled:bool }

#[derive(Debug, Clone)]
pub struct Marker(pub String);   // generic tag component

// ═══ WORLD ════════════════════════════════════════════════════════
pub struct World {
    // Entity management
    entities:         Vec<Option<u32>>,  // generation per slot
    free_slots:       Vec<u32>,
    entity_count:     u32,

    // Component storage — TypeId → storage
    storages:         HashMap<TypeId, ComponentVec>,

    // Named entity lookup
    named:            HashMap<String, Entity>,

    // Groups/tags
    groups:           HashMap<String, HashSet<u32>>,

    // Stats
    pub total_spawned:  u64,
    pub total_destroyed:u64,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities:Vec::new(), free_slots:Vec::new(), entity_count:0,
            storages:HashMap::new(), named:HashMap::new(), groups:HashMap::new(),
            total_spawned:0, total_destroyed:0,
        }
    }

    // ── Entity lifecycle ──────────────────────────────────────────
    pub fn spawn(&mut self) -> Entity {
        let (id, gen) = if let Some(slot) = self.free_slots.pop() {
            let gen = self.entities[slot as usize].map(|g|g+1).unwrap_or(1);
            self.entities[slot as usize] = Some(gen);
            (slot, gen)
        } else {
            let id = self.entities.len() as u32;
            self.entities.push(Some(0));
            (id, 0)
        };
        self.entity_count += 1;
        self.total_spawned += 1;
        Entity::new(id, gen)
    }

    pub fn spawn_with_name(&mut self, name:&str) -> Entity {
        let e = self.spawn();
        self.insert(e, Name::new(name));
        self.named.insert(name.to_string(), e);
        e
    }

    pub fn despawn(&mut self, entity:Entity) -> bool {
        let id = entity.id as usize;
        if id >= self.entities.len() { return false; }
        if self.entities[id] != Some(entity.generation) { return false; }
        self.entities[id] = None;
        self.free_slots.push(entity.id);
        // Remove all components
        for storage in self.storages.values_mut() { storage.remove(entity.id); }
        // Remove from groups
        for group in self.groups.values_mut() { group.remove(&entity.id); }
        self.entity_count -= 1;
        self.total_destroyed += 1;
        true
    }

    pub fn is_alive(&self, entity:Entity) -> bool {
        let id = entity.id as usize;
        id < self.entities.len() && self.entities[id] == Some(entity.generation)
    }

    // ── Component operations ──────────────────────────────────────
    pub fn insert<C:Component>(&mut self, entity:Entity, component:C) {
        if !self.is_alive(entity) { return; }
        let storage = self.storages.entry(TypeId::of::<C>()).or_insert_with(ComponentVec::new);
        storage.insert(entity.id, Box::new(component));
    }

    pub fn remove<C:Component>(&mut self, entity:Entity) -> bool {
        if let Some(storage) = self.storages.get_mut(&TypeId::of::<C>()) {
            storage.remove(entity.id)
        } else { false }
    }

    pub fn get<C:Component>(&self, entity:Entity) -> Option<&C> {
        if !self.is_alive(entity) { return None; }
        self.storages.get(&TypeId::of::<C>())
            .and_then(|s| s.get(entity.id))
            .and_then(|c| c.downcast_ref::<C>())
    }

    pub fn get_mut<C:Component>(&mut self, entity:Entity) -> Option<&mut C> {
        if !self.is_alive(entity) { return None; }
        self.storages.get_mut(&TypeId::of::<C>())
            .and_then(|s| s.get_mut(entity.id))
            .and_then(|c| c.downcast_mut::<C>())
    }

    pub fn has<C:Component>(&self, entity:Entity) -> bool {
        self.storages.get(&TypeId::of::<C>())
            .map(|s| s.has(entity.id))
            .unwrap_or(false)
    }

    // ── Queries ───────────────────────────────────────────────────
    pub fn query<C:Component>(&self) -> Vec<(Entity, &C)> {
        let Some(storage) = self.storages.get(&TypeId::of::<C>()) else { return Vec::new() };
        storage.entity_ids().iter().filter_map(|&id| {
            let gen = self.entities.get(id as usize).and_then(|&g|g)?;
            let e   = Entity::new(id, gen);
            let c   = storage.get(id)?.downcast_ref::<C>()?;
            Some((e, c))
        }).collect()
    }

    pub fn query_entities<C:Component>(&self) -> Vec<Entity> {
        let Some(storage) = self.storages.get(&TypeId::of::<C>()) else { return Vec::new() };
        storage.entity_ids().iter().filter_map(|&id| {
            let gen = self.entities.get(id as usize).and_then(|&g|g)?;
            Some(Entity::new(id, gen))
        }).collect()
    }

    // ── Named entities ────────────────────────────────────────────
    pub fn find_by_name(&self, name:&str) -> Option<Entity> { self.named.get(name).copied() }

    // ── Groups ────────────────────────────────────────────────────
    pub fn add_to_group(&mut self, entity:Entity, group:&str) {
        self.groups.entry(group.to_string()).or_default().insert(entity.id);
    }
    pub fn remove_from_group(&mut self, entity:Entity, group:&str) {
        if let Some(g) = self.groups.get_mut(group) { g.remove(&entity.id); }
    }
    pub fn entities_in_group(&self, group:&str) -> Vec<Entity> {
        self.groups.get(group).map(|g| {
            g.iter().filter_map(|&id| {
                let gen = self.entities.get(id as usize).and_then(|&g|g)?;
                Some(Entity::new(id, gen))
            }).collect()
        }).unwrap_or_default()
    }
    pub fn in_group(&self, entity:Entity, group:&str) -> bool {
        self.groups.get(group).map(|g| g.contains(&entity.id)).unwrap_or(false)
    }

    // ── Stats ─────────────────────────────────────────────────────
    pub fn entity_count(&self) -> u32 { self.entity_count }
    pub fn component_type_count(&self) -> usize { self.storages.len() }
    pub fn component_count<C:Component>(&self) -> usize {
        self.storages.get(&TypeId::of::<C>()).map(|s|s.len()).unwrap_or(0)
    }
}

impl Default for World { fn default() -> Self { Self::new() } }

// ═══ SYSTEM TRAIT ═════════════════════════════════════════════════
pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world:&mut World, delta:f32);
    fn enabled(&self) -> bool { true }
    fn fixed_update(&self) -> bool { false }
}

pub struct SystemRunner {
    pub systems: Vec<Box<dyn System>>,
}
impl SystemRunner {
    pub fn new() -> Self { Self { systems:Vec::new() } }
    pub fn add<S:System+'static>(&mut self, s:S) { self.systems.push(Box::new(s)); }
    pub fn run(&mut self, world:&mut World, delta:f32) {
        for sys in &mut self.systems {
            if sys.enabled() && !sys.fixed_update() { sys.run(world, delta); }
        }
    }
    pub fn run_fixed(&mut self, world:&mut World, delta:f32) {
        for sys in &mut self.systems {
            if sys.enabled() && sys.fixed_update() { sys.run(world, delta); }
        }
    }
}
