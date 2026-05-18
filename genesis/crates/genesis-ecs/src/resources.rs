//! Global resources (singletons), command queue, systems

use std::any::{Any, TypeId};
use std::collections::HashMap;
use anyhow::Result;

// ─── Resources ───────────────────────────────────────────────────────────────

/// Global singleton data (not attached to any entity)
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.map.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.map.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
        self.map.remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast().ok())
            .map(|b| *b)
    }

    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }
}

// ─── Commands ─────────────────────────────────────────────────────────────

/// A deferred world operation
pub trait Command: Send + 'static {
    fn apply(self: Box<Self>, world: &mut crate::world::World);
}

pub struct CommandQueue {
    commands: Vec<Box<dyn Command>>,
}

impl CommandQueue {
    pub fn new() -> Self { Self { commands: Vec::new() } }

    pub fn push<C: Command>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    pub fn drain(&mut self) -> Vec<Box<dyn Command>> {
        std::mem::take(&mut self.commands)
    }

    pub fn is_empty(&self) -> bool { self.commands.is_empty() }
}

/// Spawn entity command
pub struct SpawnCommand {
    pub spawn_fn: Box<dyn FnOnce(&mut crate::world::World) + Send>,
}

impl Command for SpawnCommand {
    fn apply(self: Box<Self>, world: &mut crate::world::World) {
        (self.spawn_fn)(world);
    }
}

/// Despawn entity command
pub struct DespawnCommand(pub crate::world::Entity);
impl Command for DespawnCommand {
    fn apply(self: Box<Self>, world: &mut crate::world::World) {
        world.despawn(self.0);
    }
}
