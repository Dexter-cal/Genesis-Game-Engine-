use genesis_ecs::Entity;

pub struct CyberPlayer {
    pub health: f32,
    pub energy: f32,
}

impl CyberPlayer {
    pub fn new() -> Self {
        Self { health: 100.0, energy: 50.0 }
    }
}
