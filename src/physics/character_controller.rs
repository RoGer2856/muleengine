use std::time::Instant;

use muleengine::bytifex_utils::{
    containers::object_pool::ObjectPoolIndex,
    sync::{types::ArcRwLock, usage_counter::UsageCounter},
};
use rapier3d::{
    control::{
        CharacterAutostep, CharacterLength as RapierCharacterLength, KinematicCharacterController,
    },
    prelude::{nalgebra::*, ColliderShape as RapierColliderShape, UnitVector},
};
use vek::Vec3;

use super::{collider::ColliderShape, Rapier3dPhysicsEngine};

pub enum CharacterLength {
    Absolute(f32),
    Relative(f32),
}

impl CharacterLength {
    fn as_rapier_character_length(&self) -> RapierCharacterLength {
        match self {
            CharacterLength::Absolute(value) => RapierCharacterLength::Absolute(*value),
            CharacterLength::Relative(value) => RapierCharacterLength::Relative(*value),
        }
    }
}

pub(super) struct CharacterController {
    pub(super) last_update_time: Instant,
    pub(super) predicted_next_update_time: Instant,
    pub(super) previous_position: Vec3<f32>,

    pub(super) character_controller: KinematicCharacterController,
    pub(super) position: Vec3<f32>,
    pub(super) velocity: Vec3<f32>,
    pub(super) mass: f32,
    pub(super) shape: RapierColliderShape,
    pub(super) grounded: bool,
    pub(super) falling_velocity: Vec3<f32>,
    pub(super) gravity: Vec3<f32>,
}

impl CharacterController {
    pub fn set_gravity(&mut self, gravity: Vec3<f32>) {
        self.gravity = gravity;
    }

    pub fn set_velocity(&mut self, velocity: Vec3<f32>) {
        self.velocity = velocity;
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.position = position;
        self.previous_position = position;
    }

    pub fn set_interpolated_position(&mut self, position: Vec3<f32>) {
        self.position = position;
    }

    pub fn set_collider_shape(&mut self, collider_shape: ColliderShape) {
        self.shape = collider_shape.as_rapier_collider_shape();
    }

    pub fn set_margin(&mut self, margin: CharacterLength) {
        self.character_controller.offset = margin.as_rapier_character_length();
    }

    pub fn set_up_vector(&mut self, up: Vec3<f32>) {
        self.character_controller.up = UnitVector::new_normalize(Vector3::new(up.x, up.y, up.z));
    }

    pub fn set_max_slope_climb_angle(&mut self, degree: f32) {
        self.character_controller.max_slope_climb_angle = degree.to_radians();
    }

    pub fn set_min_slope_slide_angle(&mut self, degree: f32) {
        self.character_controller.min_slope_slide_angle = degree.to_radians();
    }

    pub fn disable_autostep(&mut self) {
        self.character_controller.autostep = None;
    }

    pub fn set_autostep(&mut self, include_dynamic_bodies: bool, max_step_height: CharacterLength) {
        self.character_controller.autostep = Some(CharacterAutostep {
            max_height: max_step_height.as_rapier_character_length(),
            min_width: RapierCharacterLength::Relative(1.0),
            include_dynamic_bodies,
        });
    }

    pub fn disable_snap_to_ground(&mut self) {
        self.character_controller.snap_to_ground = None;
    }

    pub fn set_snap_to_ground(&mut self, max_snap_height: CharacterLength) {
        self.character_controller.snap_to_ground =
            Some(max_snap_height.as_rapier_character_length());
    }
}

pub struct CharacterControllerBuilder {
    character_controller: CharacterController,
}

impl CharacterControllerBuilder {
    pub(super) fn new(collider_shape: ColliderShape, gravity: Vec3<f32>, now: Instant) -> Self {
        let mut character_controller = CharacterController {
            last_update_time: now,
            predicted_next_update_time: now,

            character_controller: KinematicCharacterController::default(),
            position: Vec3::zero(),
            previous_position: Vec3::zero(),
            velocity: Vec3::zero(),
            mass: 0.0,
            shape: collider_shape.as_rapier_collider_shape(),
            grounded: false,
            gravity,
            falling_velocity: Vec3::zero(),
        };

        character_controller.set_margin(CharacterLength::Absolute(0.01));
        character_controller.set_up_vector(Vec3::unit_y());
        character_controller.set_max_slope_climb_angle(35.0);
        character_controller.set_min_slope_slide_angle(45.0);
        character_controller.disable_autostep();
        character_controller.set_snap_to_ground(CharacterLength::Absolute(0.3));

        Self {
            character_controller,
        }
    }

    pub fn gravity(mut self, gravity: Vec3<f32>) -> Self {
        self.character_controller.set_gravity(gravity);
        self
    }

    pub fn mass(mut self, mass: f32) -> Self {
        self.character_controller.set_mass(mass);
        self
    }

    pub fn position(mut self, position: Vec3<f32>) -> Self {
        self.character_controller.set_position(position);
        self
    }

    pub fn collider_shape(mut self, collider_shape: ColliderShape) -> Self {
        self.character_controller.set_collider_shape(collider_shape);
        self
    }

    pub fn margin(mut self, margin: CharacterLength) -> Self {
        self.character_controller.set_margin(margin);
        self
    }

    pub fn up_vector(mut self, up: Vec3<f32>) -> Self {
        self.character_controller.set_up_vector(up);
        self
    }

    pub fn max_slope_climb_angle(mut self, degree: f32) -> Self {
        self.character_controller.set_max_slope_climb_angle(degree);
        self
    }

    pub fn min_slope_slide_angle(mut self, degree: f32) -> Self {
        self.character_controller.set_min_slope_slide_angle(degree);
        self
    }

    pub fn disable_autostep(mut self) -> Self {
        self.character_controller.disable_autostep();
        self
    }

    pub fn autostep(
        mut self,
        include_dynamic_bodies: bool,
        max_step_height: CharacterLength,
    ) -> Self {
        self.character_controller
            .set_autostep(include_dynamic_bodies, max_step_height);
        self
    }

    pub fn disable_snap_to_ground(mut self) -> Self {
        self.character_controller.disable_snap_to_ground();
        self
    }

    pub fn snap_to_ground(mut self, max_snap_height: CharacterLength) -> Self {
        self.character_controller
            .set_snap_to_ground(max_snap_height);
        self
    }

    pub fn build(self, physics_engine: &mut Rapier3dPhysicsEngine) -> CharacterControllerHandler {
        physics_engine.add_character_controller(self.character_controller)
    }
}

#[derive(Clone)]
pub struct CharacterControllerHandler {
    pub(super) usage_counter: UsageCounter,
    pub(super) object_pool_index: ObjectPoolIndex,
    pub(super) character_controller: ArcRwLock<CharacterController>,
    pub(super) to_be_dropped_character_controllers: ArcRwLock<Vec<ObjectPoolIndex>>,
}

impl Drop for CharacterControllerHandler {
    fn drop(&mut self) {
        if self.usage_counter.is_this_the_last() {
            self.to_be_dropped_character_controllers
                .write()
                .push(self.object_pool_index);
        }
    }
}

impl CharacterControllerHandler {
    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.character_controller.write().set_position(position);
    }

    pub fn set_interpolated_position(&mut self, position: Vec3<f32>) {
        self.character_controller
            .write()
            .set_interpolated_position(position);
    }

    pub fn set_gravity(&mut self, gravity: Vec3<f32>) {
        self.character_controller.write().set_gravity(gravity);
    }

    pub fn set_velocity(&mut self, velocity: Vec3<f32>) {
        self.character_controller.write().set_velocity(velocity);
    }

    pub fn get_velocity(&self) -> Vec3<f32> {
        self.character_controller.read().velocity
    }

    pub fn get_position(&self) -> Vec3<f32> {
        self.character_controller.read().position
    }

    pub fn get_interpolated_position(&self, now: &Instant) -> Vec3<f32> {
        let character_controller = self.character_controller.read();

        let time_elapsed_since_last_update_secs = now
            .duration_since(character_controller.last_update_time)
            .as_secs_f32();
        let update_interval_duration_secs = character_controller
            .predicted_next_update_time
            .duration_since(character_controller.last_update_time)
            .as_secs_f32();
        let q = time_elapsed_since_last_update_secs / update_interval_duration_secs;

        character_controller.previous_position * (1.0 - q) + character_controller.position * q
    }
}
