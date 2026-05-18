//! The World — central ECS data store.
//! All entities and components live here.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use anyhow::{Result, anyhow};

use crate::component::*;
use crate::resources::Resources;
use crate::commands::CommandQueue;

/// Simple incrementing entity ID (u64 for uniqueness)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(pub u64);

static NEXT_ENTITY_ID: AtomicU64 = AtomicU64::new(1);

impl Entity {
    pub fn new() -> Self {
        Self(NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed))
    }
    pub fn id(&self) -> u64 { self.0 }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// Type-erased component storage
trait ComponentStorage: Send + Sync + Any {
    fn remove(&mut self, entity: Entity);
    fn has(&self, entity: Entity) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Dense storage for a single component type
struct Storage<T: Component> {
    components: HashMap<Entity, T>,
}

impl<T: Component> Storage<T> {
    fn new() -> Self {
        Self { components: HashMap::new() }
    }

    fn insert(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&entity)
    }
}

impl<T: Component + 'static> ComponentStorage for Storage<T> {
    fn remove(&mut self, entity: Entity) {
        self.components.remove(&entity);
    }
    fn has(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity)
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// The ECS World — holds all entities and their components
pub struct World {
    /// All component storages, keyed by TypeId
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    /// All active entities
    entities: std::collections::HashSet<Entity>,
    /// Entities pending removal (removed end of frame)
    pending_despawn: Vec<Entity>,
    /// Global resources (singleton data)
    pub resources: Resources,
    /// Command queue for deferred operations
    pub commands: CommandQueue,
    /// Entity counter (for debug info)
    entity_count: u64,
}

impl World {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            entities: std::collections::HashSet::new(),
            pending_despawn: Vec::new(),
            resources: Resources::new(),
            commands: CommandQueue::new(),
            entity_count: 0,
        }
    }

    /// Create a new entity with no components
    pub fn spawn_empty(&mut self) -> Entity {
        let entity = Entity::new();
        self.entities.insert(entity);
        self.entity_count += 1;
        entity
    }

    /// Create a new entity and insert components from a bundle
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = self.spawn_empty();
        bundle.insert_into(self, entity);
        entity
    }

    /// Insert a component onto an entity
    pub fn insert<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(Storage::<T>::new()));

        let storage = storage
            .as_any_mut()
            .downcast_mut::<Storage<T>>()
            .expect("Component type mismatch");

        storage.insert(entity, component);
    }

    /// Get a component reference
    pub fn get<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get(&type_id)?;
        let storage = storage.as_any().downcast_ref::<Storage<T>>()?;
        storage.get(entity)
    }

    /// Get a mutable component reference
    pub fn get_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self.storages.get_mut(&type_id)?;
        let storage = storage.as_any_mut().downcast_mut::<Storage<T>>()?;
        storage.get_mut(entity)
    }

    /// Remove a component from an entity
    pub fn remove<T: Component + 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storages.get_mut(&type_id) {
            storage.remove(entity);
        }
    }

    /// Check if entity has a component
    pub fn has<T: Component + 'static>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id)
            .map(|s| s.has(entity))
            .unwrap_or(false)
    }

    /// Schedule entity for removal at end of frame
    pub fn despawn(&mut self, entity: Entity) {
        self.pending_despawn.push(entity);
    }

    /// Actually remove pending despawn entities (call end of frame)
    pub fn flush_despawn(&mut self) {
        let to_remove: Vec<Entity> = self.pending_despawn.drain(..).collect();
        for entity in to_remove {
            self.entities.remove(&entity);
            // Remove from all storages
            for storage in self.storages.values_mut() {
                storage.remove(entity);
            }
            self.entity_count -= 1;
        }
    }

    /// Apply all queued commands
    pub fn flush_commands(&mut self) {
        let commands = self.commands.drain();
        for cmd in commands {
            cmd.apply(self);
        }
    }

    /// Iterate all entities with Position component
    pub fn query_positions(&self) -> Vec<(Entity, &Position)> {
        let type_id = TypeId::of::<Position>();
        match self.storages.get(&type_id) {
            None => vec![],
            Some(storage) => {
                let storage = storage.as_any().downcast_ref::<Storage<Position>>().unwrap();
                storage.components.iter()
                    .filter(|(e, _)| self.entities.contains(e))
                    .map(|(e, c)| (*e, c))
                    .collect()
            }
        }
    }

    /// Query single component type immutably
    pub fn query<T: Component + 'static>(&self) -> Vec<(Entity, &T)> {
        let type_id = TypeId::of::<T>();
        match self.storages.get(&type_id) {
            None => vec![],
            Some(storage) => {
                let storage = storage.as_any().downcast_ref::<Storage<T>>().unwrap();
                storage.components.iter()
                    .filter(|(e, _)| self.entities.contains(e))
                    .map(|(e, c)| (*e, c))
                    .collect()
            }
        }
    }

    /// Query two component types together (inner join)
    pub fn query2<A: Component + 'static, B: Component + 'static>(&self)
        -> Vec<(Entity, &A, &B)>
    {
        let type_a = TypeId::of::<A>();
        let type_b = TypeId::of::<B>();

        let storage_a = match self.storages.get(&type_a) {
            Some(s) => s.as_any().downcast_ref::<Storage<A>>().unwrap(),
            None => return vec![],
        };
        let storage_b = match self.storages.get(&type_b) {
            Some(s) => s.as_any().downcast_ref::<Storage<B>>().unwrap(),
            None => return vec![],
        };

        storage_a.components.iter()
            .filter(|(e, _)| self.entities.contains(e))
            .filter_map(|(e, a)| {
                storage_b.get(*e).map(|b| (*e, a, b))
            })
            .collect()
    }

    /// Get all entities with a given tag
    pub fn entities_with_tag(&self, tag: &str) -> Vec<Entity> {
        let type_id = TypeId::of::<Tags>();
        match self.storages.get(&type_id) {
            None => vec![],
            Some(storage) => {
                let storage = storage.as_any().downcast_ref::<Storage<Tags>>().unwrap();
                storage.components.iter()
                    .filter(|(e, tags)| self.entities.contains(e) && tags.has(tag))
                    .map(|(e, _)| *e)
                    .collect()
            }
        }
    }

    /// Find entities within radius of a position
    pub fn entities_in_radius(&self, center: [f32; 3], radius: f32) -> Vec<Entity> {
        let r2 = radius * radius;
        let pos_type = TypeId::of::<Position>();

        match self.storages.get(&pos_type) {
            None => vec![],
            Some(storage) => {
                let storage = storage.as_any().downcast_ref::<Storage<Position>>().unwrap();
                storage.components.iter()
                    .filter(|(e, _)| self.entities.contains(e))
                    .filter(|(_, pos)| {
                        let dx = pos.x - center[0];
                        let dy = pos.y - center[1];
                        let dz = pos.z - center[2];
                        dx*dx + dy*dy + dz*dz <= r2
                    })
                    .map(|(e, _)| *e)
                    .collect()
            }
        }
    }

    pub fn entity_count(&self) -> u64 { self.entity_count }
    pub fn is_alive(&self, entity: Entity) -> bool { self.entities.contains(&entity) }
    pub fn all_entities(&self) -> &std::collections::HashSet<Entity> { &self.entities }
}

/// Bundle — a collection of components to insert at once
pub trait Bundle: Send + 'static {
    fn insert_into(self, world: &mut World, entity: Entity);
}

/// Implement Bundle for tuples of components
macro_rules! impl_bundle {
    ($($T:ident),+) => {
        impl<$($T: Component + 'static),+> Bundle for ($($T,)+) {
            fn insert_into(self, world: &mut World, entity: Entity) {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                $(world.insert(entity, $T);)+
            }
        }
    }
}

impl_bundle!(A);
impl_bundle!(A, B);
impl_bundle!(A, B, C);
impl_bundle!(A, B, C, D);
impl_bundle!(A, B, C, D, E);
impl_bundle!(A, B, C, D, E, F);
impl_bundle!(A, B, C, D, E, F, G);
impl_bundle!(A, B, C, D, E, F, G, H);
impl_bundle!(A, B, C, D, E, F, G, H, I);
impl_bundle!(A, B, C, D, E, F, G, H, I, J);
