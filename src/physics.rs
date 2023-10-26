use std::time::Duration;

use method_taskifier::{
    method_taskifier_impl,
    prelude::{OptionInspector, ResultInspector},
};
use muleengine::{
    application_runner::ApplicationContext,
    bytifex_utils::sync::{app_loop_state::AppLoopStateWatcher, types::ArcRwLock},
};
use parking_lot::RwLock;
use rapier3d::prelude::*;
use tokio::time::{interval, MissedTickBehavior};
use vek::{Quaternion, Vec3};

pub type Rapier3dPhysicsEngineService = RwLock<Rapier3dPhysicsEngine>;

pub struct RigidBodyDescriptor {
    rigid_body_handle: RigidBodyHandle,
}

#[derive(Clone)]
pub struct Rapier3dObjectsState {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
}

pub struct Rapier3dPhysicsEngine {
    integration_parameters: IntegrationParameters,

    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,

    state: Rapier3dObjectsState,

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

#[method_taskifier_impl(module_name = physics_decoupler)]
impl Rapier3dPhysicsEngine {
    fn new() -> Self {
        Self::from_objects_state(Rapier3dObjectsState {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
        })
    }

    fn from_objects_state(state: Rapier3dObjectsState) -> Self {
        let mut ret = Self {
            integration_parameters: IntegrationParameters::default(),

            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),

            state,

            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        };

        /* Create the ground. */
        // todo!("remove ground")
        let collider = ColliderBuilder::cuboid(1000.0, 0.1, 1000.0).build();
        ret.state.collider_set.insert(collider);

        ret
    }

    pub fn create_sphere_rigid_body(
        &mut self,
        position: Vec3<f32>,
        radius: f32,
    ) -> RigidBodyDescriptor {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x, position.y, position.z])
            .build();
        let collider = ColliderBuilder::ball(radius).restitution(0.0).build();
        let rigid_body_handle = self.state.rigid_body_set.insert(rigid_body);
        self.state.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.state.rigid_body_set,
        );

        RigidBodyDescriptor { rigid_body_handle }
    }

    pub fn create_box_rigid_body(
        &mut self,
        position: Vec3<f32>,
        dimensions: Vec3<f32>,
    ) -> RigidBodyDescriptor {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![position.x, position.y, position.z])
            .build();
        let collider = ColliderBuilder::cuboid(dimensions.x, dimensions.y, dimensions.z)
            .restitution(0.0)
            .build();
        let rigid_body_handle = self.state.rigid_body_set.insert(rigid_body);
        self.state.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.state.rigid_body_set,
        );

        RigidBodyDescriptor { rigid_body_handle }
    }

    async fn run(
        app_loop_state_watcher: AppLoopStateWatcher,
        receiver: ArcRwLock<Rapier3dPhysicsEngine>,
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
                    receiver.write().step(interval_secs);
                }
            }

            // let end = Instant::now();
            // delta_time_secs = (end - start).as_secs_f32();
        }
    }

    fn step(&mut self, delta_time_in_secs: f32) {
        let gravity = vector![0.0, -9.81, 0.0];

        self.integration_parameters.dt = delta_time_in_secs;

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.state.rigid_body_set,
            &mut self.state.collider_set,
            &mut self.state.impulse_joint_set,
            &mut self.state.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}

impl Rapier3dObjectsState {
    pub fn get_transform_of_rigidbody(
        &mut self,
        rigid_body_descriptor: &RigidBodyDescriptor,
    ) -> (Vec3<f32>, Quaternion<f32>) {
        let rigid_body = &self.rigid_body_set[rigid_body_descriptor.rigid_body_handle];
        let position = rigid_body.translation();
        let rotation = rigid_body.rotation().as_vector();
        (
            Vec3::new(position.x, position.y, position.z),
            Quaternion::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w),
        )
    }
}
