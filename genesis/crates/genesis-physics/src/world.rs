//! Physics world — wraps Rapier3D with Genesis conventions.

use rapier3d::prelude::*;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use anyhow::Result;
use tracing::{debug, warn};

use genesis_math::vec3::Vec3;
use genesis_core::events::{EventBus, GameEvent, EventPriority};

/// Maps Genesis entity string IDs to Rapier handles
#[derive(Default)]
pub struct PhysicsHandles {
    pub rigid_bodies:    HashMap<String, RigidBodyHandle>,
    pub colliders:       HashMap<String, ColliderHandle>,
    /// Reverse map: Rapier body handle → entity ID
    pub body_to_entity:  HashMap<RigidBodyHandle, String>,
    pub coll_to_entity:  HashMap<ColliderHandle, String>,
}

/// The main physics simulation state
pub struct PhysicsWorld {
    pub pipeline:             PhysicsPipeline,
    pub gravity:              rapier3d::math::Vector<f32>,
    pub integration_params:   IntegrationParameters,
    pub islands:              IslandManager,
    pub broad_phase:          DefaultBroadPhase,
    pub narrow_phase:         NarrowPhase,
    pub bodies:               RigidBodySet,
    pub colliders:            ColliderSet,
    pub impulse_joints:       ImpulseJointSet,
    pub multibody_joints:     MultibodyJointSet,
    pub ccd_solver:           CCDSolver,
    pub query_pipeline:       QueryPipeline,
    pub handles:              PhysicsHandles,
    /// Fixed-timestep accumulator
    accumulator:              f32,
    /// Fixed timestep (1/120 s by default)
    pub fixed_dt:             f32,
    /// Enable debug drawing
    pub debug_draw:           bool,
    /// Pending collision events from hooks
    collision_events:         Arc<RwLock<Vec<CollisionEventData>>>,
    /// Total simulation steps run
    pub step_count:           u64,
}

/// A collision event produced by the physics engine
#[derive(Debug, Clone)]
pub struct CollisionEventData {
    pub entity_a:      String,
    pub entity_b:      String,
    pub contact_point: Vec3,
    pub normal:        Vec3,
    pub impulse:       f32,
    pub started:       bool, // true = enter, false = exit
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3, fixed_hz: f64) -> Self {
        let mut integration_params = IntegrationParameters::default();
        integration_params.dt = (1.0 / fixed_hz) as f32;

        Self {
            pipeline:           PhysicsPipeline::new(),
            gravity:            rapier3d::math::vector![gravity.x, gravity.y, gravity.z],
            integration_params,
            islands:            IslandManager::new(),
            broad_phase:        DefaultBroadPhase::new(),
            narrow_phase:       NarrowPhase::new(),
            bodies:             RigidBodySet::new(),
            colliders:          ColliderSet::new(),
            impulse_joints:     ImpulseJointSet::new(),
            multibody_joints:   MultibodyJointSet::new(),
            ccd_solver:         CCDSolver::new(),
            query_pipeline:     QueryPipeline::new(),
            handles:            PhysicsHandles::default(),
            accumulator:        0.0,
            fixed_dt:           (1.0 / fixed_hz) as f32,
            debug_draw:         false,
            collision_events:   Arc::new(RwLock::new(Vec::new())),
            step_count:         0,
        }
    }

    /// Step the physics simulation, called every game frame.
    /// Uses fixed-timestep with accumulator for stability.
    pub fn step(&mut self, delta: f32, game_bus: &EventBus<GameEvent>) {
        self.accumulator += delta;

        // Run as many fixed steps as the accumulator allows
        let max_steps = 4; // Prevent spiral of death
        let mut steps_run = 0;

        while self.accumulator >= self.fixed_dt && steps_run < max_steps {
            self.step_once();
            self.accumulator -= self.fixed_dt;
            steps_run += 1;
        }

        // Update query pipeline for raycasts
        self.query_pipeline.update(&self.colliders);

        // Flush collision events to game bus
        self.flush_collision_events(game_bus);
    }

    fn step_once(&mut self) {
        let event_handler = CollisionHook {
            events: Arc::clone(&self.collision_events),
            handles: &self.handles,
        };

        self.pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &event_handler,
            &(),
        );
        self.step_count += 1;
    }

    fn flush_collision_events(&self, bus: &EventBus<GameEvent>) {
        let events: Vec<CollisionEventData> = {
            let mut lock = self.collision_events.write();
            std::mem::take(&mut *lock)
        };

        for ev in events {
            if ev.started {
                bus.publish(
                    GameEvent::CollisionEnter {
                        entity_a: ev.entity_a,
                        entity_b: ev.entity_b,
                        contact_point: ev.contact_point.to_array(),
                        normal: ev.normal.to_array(),
                        impulse: ev.impulse,
                    },
                    EventPriority::Critical,
                    Some("physics".to_string()),
                );
            } else {
                bus.publish(
                    GameEvent::CollisionExit {
                        entity_a: ev.entity_a,
                        entity_b: ev.entity_b,
                    },
                    EventPriority::High,
                    Some("physics".to_string()),
                );
            }
        }
    }

    // ─── Body Management ─────────────────────────────────────────────────

    /// Add a rigid body to the simulation
    pub fn add_rigid_body(
        &mut self,
        entity_id: &str,
        body: RigidBody,
        collider: Collider,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body_handle = self.bodies.insert(body);
        let coll_handle = self.colliders.insert_with_parent(
            collider,
            body_handle,
            &mut self.bodies,
        );

        self.handles.rigid_bodies.insert(entity_id.to_string(), body_handle);
        self.handles.colliders.insert(entity_id.to_string(), coll_handle);
        self.handles.body_to_entity.insert(body_handle, entity_id.to_string());
        self.handles.coll_to_entity.insert(coll_handle, entity_id.to_string());

        debug!("Added rigid body for entity '{}'", entity_id);
        (body_handle, coll_handle)
    }

    /// Add a static collider (no rigid body — terrain, walls)
    pub fn add_static_collider(&mut self, entity_id: &str, collider: Collider) -> ColliderHandle {
        let handle = self.colliders.insert(collider);
        self.handles.colliders.insert(entity_id.to_string(), handle);
        self.handles.coll_to_entity.insert(handle, entity_id.to_string());
        handle
    }

    /// Remove all physics objects for an entity
    pub fn remove_entity(&mut self, entity_id: &str) {
        if let Some(body_handle) = self.handles.rigid_bodies.remove(entity_id) {
            self.bodies.remove(
                body_handle,
                &mut self.islands,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
            self.handles.body_to_entity.remove(&body_handle);
        }
        self.handles.colliders.remove(entity_id);
    }

    /// Get current position/rotation of an entity from physics
    pub fn get_transform(&self, entity_id: &str) -> Option<(Vec3, [f32; 4])> {
        let handle = self.handles.rigid_bodies.get(entity_id)?;
        let body = self.bodies.get(*handle)?;
        let pos = body.translation();
        let rot = body.rotation();
        Some((
            Vec3::new(pos.x, pos.y, pos.z),
            [rot.i, rot.j, rot.k, rot.w],
        ))
    }

    /// Apply an impulse to an entity
    pub fn apply_impulse(&mut self, entity_id: &str, impulse: Vec3) {
        if let Some(&handle) = self.handles.rigid_bodies.get(entity_id) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.apply_impulse(
                    rapier3d::math::vector![impulse.x, impulse.y, impulse.z],
                    true,
                );
            }
        }
    }

    /// Apply a force to an entity
    pub fn apply_force(&mut self, entity_id: &str, force: Vec3) {
        if let Some(&handle) = self.handles.rigid_bodies.get(entity_id) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.add_force(
                    rapier3d::math::vector![force.x, force.y, force.z],
                    true,
                );
            }
        }
    }

    /// Teleport an entity (set position directly)
    pub fn set_position(&mut self, entity_id: &str, position: Vec3) {
        if let Some(&handle) = self.handles.rigid_bodies.get(entity_id) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.set_translation(
                    rapier3d::math::vector![position.x, position.y, position.z],
                    true,
                );
            }
        }
    }

    /// Set the velocity of an entity
    pub fn set_velocity(&mut self, entity_id: &str, velocity: Vec3) {
        if let Some(&handle) = self.handles.rigid_bodies.get(entity_id) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.set_linvel(
                    rapier3d::math::vector![velocity.x, velocity.y, velocity.z],
                    true,
                );
            }
        }
    }

    /// Set gravity for the whole world
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = rapier3d::math::vector![gravity.x, gravity.y, gravity.z];
    }

    /// Set gravity scale for a specific entity
    pub fn set_gravity_scale(&mut self, entity_id: &str, scale: f32) {
        if let Some(&handle) = self.handles.rigid_bodies.get(entity_id) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.set_gravity_scale(scale, true);
            }
        }
    }

    pub fn body_count(&self) -> usize { self.bodies.len() }
    pub fn collider_count(&self) -> usize { self.colliders.len() }
}

/// Rapier collision event hook
struct CollisionHook<'a> {
    events: Arc<RwLock<Vec<CollisionEventData>>>,
    handles: &'a PhysicsHandles,
}

impl<'a> EventHandler for CollisionHook<'a> {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: rapier3d::geometry::CollisionEvent,
        contact_pair: Option<&rapier3d::geometry::ContactPair>,
    ) {
        let (h1, h2, started) = match event {
            rapier3d::geometry::CollisionEvent::Started(h1, h2, _) => (h1, h2, true),
            rapier3d::geometry::CollisionEvent::Stopped(h1, h2, _) => (h1, h2, false),
        };

        let entity_a = self.handles.coll_to_entity.get(&h1).cloned().unwrap_or_default();
        let entity_b = self.handles.coll_to_entity.get(&h2).cloned().unwrap_or_default();

        let (contact_point, normal, impulse) = if let Some(cp) = contact_pair {
            if let Some(manifold) = cp.manifolds.first() {
                if let Some(point) = manifold.points.first() {
                    let p = point.local_p1;
                    let n = manifold.data.normal;
                    (
                        Vec3::new(p.x, p.y, p.z),
                        Vec3::new(n.x, n.y, n.z),
                        point.data.impulse,
                    )
                } else { (Vec3::ZERO, Vec3::UP, 0.0) }
            } else { (Vec3::ZERO, Vec3::UP, 0.0) }
        } else { (Vec3::ZERO, Vec3::UP, 0.0) };

        let mut events = self.events.write();
        events.push(CollisionEventData { entity_a, entity_b, contact_point, normal, impulse, started });
    }

    fn handle_contact_force_event(
        &self,
        _dt: f32,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &rapier3d::geometry::ContactPair,
        _total_force_magnitude: f32,
    ) {}
}

use genesis_math::vec3::Vec3 as MathVec3;
use std::ops::Neg;

// Helper: Vec3::UP for default normals
impl Vec3 {
    const UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
}
