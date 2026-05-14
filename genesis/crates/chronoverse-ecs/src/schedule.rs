pub struct Schedule { pub systems: Vec<Box<dyn crate::system::System>> }
impl Schedule {
    pub fn new() -> Self { Self { systems: Vec::new() } }
    pub fn add_system<S: crate::system::System + 'static>(&mut self, s: S) { self.systems.push(Box::new(s)); }
    pub async fn run(&mut self, world: &mut crate::world::World, delta: f32) {
        for sys in &mut self.systems {
            let _ = sys.run(world, delta);
        }
    }
}
