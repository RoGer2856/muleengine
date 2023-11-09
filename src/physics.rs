use std::{collections::VecDeque, sync::Arc, time::Duration};

use method_taskifier::{
    method_taskifier_impl,
    prelude::{OptionInspector, ResultInspector},
};
use muleengine::{
    application_runner::ApplicationContext,
    bytifex_utils::sync::{app_loop_state::AppLoopStateWatcher, types::ArcRwLock},
    heightmap::HeightMap,
};
use parking_lot::RwLock;
use rapier3d::prelude::{
    nalgebra::{self, *},
    BroadPhase, CCDSolver, Collider, ColliderBuilder as RapierColliderBuilder, ColliderSet,
    ColliderShape as RapierColliderShape, ImpulseJointSet, IntegrationParameters, IslandManager,
    MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodyBuilder as RapierRigidBodyBuilder,
    RigidBodyHandle as RapierRigidBodyHandle, RigidBodySet, Vector,
};
use tokio::time::{interval, Instant, MissedTickBehavior};
use vek::{Quaternion, Vec3};

pub type Rapier3dPhysicsEngineService = RwLock<Rapier3dPhysicsEngine>;

const NUMBER_OF_STORED_STATES: usize = 1;

pub struct RigidBodyHandler {
    inner_handle: RapierRigidBodyHandle,
}

#[derive(Clone)]
pub struct Rapier3dObjectsState {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
}

pub struct Rapier3dPhysicsEngine {
    last_tick_time: Instant,
    predicted_next_tick_time: Instant,

    integration_parameters: IntegrationParameters,

    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,

    current_state: Rapier3dObjectsState,
    previous_states: VecDeque<Rapier3dObjectsState>,

    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
}

pub fn run(app_context: &mut ApplicationContext) {
    let app_loop_state_watcher = app_context
        .service_container_ref()
        .get_service::<AppLoopStateWatcher>()
        .inspect_err(|e| log::error!("{e:?}"))
        .unwrap()
        .as_ref()
        .clone();

    let result = app_context
        .service_container_ref()
        .insert(RwLock::new(Rapier3dPhysicsEngine::new()));

    result.old_item.inspect(|_| {
        let error_msg = "Rapier3dPhysicsEngine already added to the service container";
        log::warn!("{error_msg}");
        panic!("{error_msg}");
    });

    let physics_engine = result.new_item.as_arc_ref().clone();

    tokio::spawn(async move {
        Rapier3dPhysicsEngine::run(app_loop_state_watcher, physics_engine).await;
    });
}

pub enum ColliderShape {
    Capsule {
        radius: f32,
        height: f32,
    },
    Cone {
        radius: f32,
        height: f32,
    },
    Cylinder {
        radius: f32,
        height: f32,
    },
    Box {
        x: f32,
        y: f32,
        z: f32,
    },
    Sphere {
        radius: f32,
    },
    Heightmap {
        heightmap: Arc<HeightMap>,
        scale: Vec3<f32>,
    },
}

pub struct ColliderBuilder {
    position: Vec3<f32>,
    shape: ColliderShape,
    is_sensor: bool,
}

impl ColliderBuilder {
    fn new(shape: ColliderShape) -> Self {
        Self {
            position: Vec3::broadcast(0.0),
            shape,
            is_sensor: false,
        }
    }

    pub fn position(mut self, position: Vec3<f32>) -> Self {
        self.position = position;
        self
    }

    pub fn is_sensor(mut self, is_sensor: bool) -> Self {
        self.is_sensor = is_sensor;
        self
    }

    pub fn build(self) -> Collider {
        let mut position_offset = Vec3::zero();
        let rapier_shape = match self.shape {
            ColliderShape::Capsule { radius, height } => {
                RapierColliderShape::capsule_y(height / 2.0, radius)
            }
            ColliderShape::Cone { radius, height } => {
                RapierColliderShape::cone(height / 2.0, radius)
            }
            ColliderShape::Cylinder { radius, height } => {
                RapierColliderShape::cylinder(height / 2.0, radius)
            }
            ColliderShape::Box { x, y, z } => {
                RapierColliderShape::cuboid(x / 2.0, y / 2.0, z / 2.0)
            }
            ColliderShape::Sphere { radius } => RapierColliderShape::ball(radius),
            ColliderShape::Heightmap { heightmap, scale } => {
                let scale = Vector::new(scale.x, scale.y, scale.z);
                let heights = DMatrix::from_fn(
                    heightmap.get_row_count(),
                    heightmap.get_column_count(),
                    |x, y| heightmap.get_height_map()[y][x],
                );
                position_offset.y = -scale.y / 2.0;
                RapierColliderShape::heightfield(heights, scale)
            }
        };

        RapierColliderBuilder::new(rapier_shape)
            .translation(vector![
                self.position.x + position_offset.x,
                self.position.y + position_offset.y,
                self.position.z + position_offset.z
            ])
            .sensor(self.is_sensor)
            .build()
    }
}

#[derive(Clone)]
pub struct RigidBody {
    pub colliders: Vec<Collider>,
    pub position: Vec3<f32>,
    pub rigid_body_type: RigidBodyType,
}

#[derive(Clone)]
pub enum RigidBodyType {
    Dynamic,
    Static,
}

pub struct RigidBodyBuilder {
    rigid_body: RigidBody,
}

impl RigidBodyBuilder {
    fn new(collider: Collider, rigid_body_type: RigidBodyType) -> Self {
        Self {
            rigid_body: RigidBody {
                colliders: vec![collider],
                position: Vec3::broadcast(0.0),
                rigid_body_type,
            },
        }
    }

    pub fn position(mut self, position: Vec3<f32>) -> Self {
        self.rigid_body.position = position;
        self
    }

    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.rigid_body.colliders.push(collider);
        self
    }

    pub fn build(self) -> RigidBody {
        self.rigid_body
    }
}

#[method_taskifier_impl(module_name = physics_decoupler)]
impl Rapier3dPhysicsEngine {
    pub fn collider_builder(&self, shape: ColliderShape) -> ColliderBuilder {
        ColliderBuilder::new(shape)
    }

    pub fn rigid_body_builder(
        &self,
        collider: Collider,
        rigid_body_type: RigidBodyType,
    ) -> RigidBodyBuilder {
        RigidBodyBuilder::new(collider, rigid_body_type)
    }

    pub fn add_rigid_body(&mut self, rigid_body: RigidBody) -> RigidBodyHandler {
        let rigid_body_builder = match rigid_body.rigid_body_type {
            RigidBodyType::Dynamic => RapierRigidBodyBuilder::dynamic(),
            RigidBodyType::Static => RapierRigidBodyBuilder::fixed(),
        };
        let rapier_rigid_body = rigid_body_builder
            .translation(vector![
                rigid_body.position.x,
                rigid_body.position.y,
                rigid_body.position.z
            ])
            .build();

        let rigid_body_handle = self.current_state.rigid_body_set.insert(rapier_rigid_body);

        for collider in rigid_body.colliders {
            self.current_state.collider_set.insert_with_parent(
                collider,
                rigid_body_handle,
                &mut self.current_state.rigid_body_set,
            );
        }

        RigidBodyHandler {
            inner_handle: rigid_body_handle,
        }
    }

    pub fn get_interpolated_transform_of_rigidbody(
        &self,
        rigid_body_handler: &RigidBodyHandler,
        now: Instant,
    ) -> Option<(Vec3<f32>, Quaternion<f32>)> {
        let previous_state = if let Some(previous_state) = self.previous_states.back() {
            previous_state
        } else {
            &self.current_state
        };

        let previous_rigid_body_transform =
            previous_state.get_transform_of_rigidbody(rigid_body_handler);
        let current_rigid_body_transform = self
            .current_state
            .get_transform_of_rigidbody(rigid_body_handler);

        let previous_rigid_body_transform = previous_rigid_body_transform
            .as_ref()
            .or(current_rigid_body_transform.as_ref());

        previous_rigid_body_transform
            .zip(current_rigid_body_transform.as_ref())
            .map(|(previous_transform, current_transform)| {
                let previous_position = previous_transform.0;
                let previous_orientation = previous_transform.1;

                let current_position = current_transform.0;
                let current_orientation = current_transform.1;

                let time_elapsed_since_last_tick_secs =
                    now.duration_since(self.last_tick_time).as_secs_f32();
                let tick_duration_secs = self
                    .predicted_next_tick_time
                    .duration_since(self.last_tick_time)
                    .as_secs_f32();
                let q = time_elapsed_since_last_tick_secs / tick_duration_secs;

                (
                    previous_position * (1.0 - q) + current_position * q,
                    Quaternion::slerp_unclamped(previous_orientation, current_orientation, q),
                )
            })
    }

    fn new() -> Self {
        Self::from_objects_state(Rapier3dObjectsState {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
        })
    }

    fn from_objects_state(state: Rapier3dObjectsState) -> Self {
        let mut previous_states = VecDeque::with_capacity(NUMBER_OF_STORED_STATES);
        previous_states.push_back(state.clone());

        let current_time = Instant::now();

        Self {
            last_tick_time: current_time,
            predicted_next_tick_time: current_time,

            integration_parameters: IntegrationParameters::default(),

            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),

            current_state: state,
            previous_states,

            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    async fn run(
        app_loop_state_watcher: AppLoopStateWatcher,
        physics_engine: ArcRwLock<Rapier3dPhysicsEngine>,
    ) {
        let interval_secs = 1.0 / 15.0;
        let mut interval = interval(Duration::from_secs_f32(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Burst);

        // let mut delta_time_secs = 0.0;

        loop {
            // let start = Instant::now();

            tokio::select! {
                _ = app_loop_state_watcher.wait_for_quit() => {
                    break;
                }
                _ = interval.tick() => {
                    // self.step(delta_time_secs);
                    physics_engine.write().step(interval_secs);
                }
            }

            // let end = Instant::now();
            // delta_time_secs = (end - start).as_secs_f32();
        }
    }

    fn step(&mut self, delta_time_in_secs: f32) {
        self.last_tick_time = Instant::now();
        self.predicted_next_tick_time =
            self.last_tick_time + Duration::from_secs_f32(delta_time_in_secs);

        let gravity = vector![0.0, -9.81, 0.0];

        self.integration_parameters.dt = delta_time_in_secs;

        if self.previous_states.len() == NUMBER_OF_STORED_STATES {
            self.previous_states.pop_front();
        }
        self.previous_states.push_back(self.current_state.clone());

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.current_state.rigid_body_set,
            &mut self.current_state.collider_set,
            &mut self.current_state.impulse_joint_set,
            &mut self.current_state.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}

impl Rapier3dObjectsState {
    pub fn get_transform_of_rigidbody(
        &self,
        rigid_body_handler: &RigidBodyHandler,
    ) -> Option<(Vec3<f32>, Quaternion<f32>)> {
        self.rigid_body_set
            .get(rigid_body_handler.inner_handle)
            .map(|rigid_body| {
                let position = rigid_body.translation();
                let rotation = rigid_body.rotation().as_vector();
                (
                    Vec3::new(position.x, position.y, position.z),
                    Quaternion::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w),
                )
            })
    }
}
