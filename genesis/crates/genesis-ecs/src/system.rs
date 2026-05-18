//! System trait and execution
use crate::world::World;
use anyhow::Result;

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World, delta: f32) -> Result<()>;
}
