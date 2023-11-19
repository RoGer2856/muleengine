use rapier3d::{
    control::{
        CharacterAutostep, CharacterLength as RapierCharacterLength, KinematicCharacterController,
    },
    prelude::{nalgebra::*, ColliderShape as RapierColliderShape, UnitVector},
};
use vek::Vec3;

use super::collider::ColliderShape;

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

pub struct CharacterController {
    pub(super) character_controller: KinematicCharacterController,
    pub(super) position: Vec3<f32>,
    pub(super) shape: RapierColliderShape,
    pub(super) grounded: bool,
}

impl CharacterController {
    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vec3<f32> {
        self.position
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
    pub(super) fn new(collider_shape: ColliderShape) -> Self {
        let mut character_controller = CharacterController {
            character_controller: KinematicCharacterController::default(),
            position: Vec3::zero(),
            shape: collider_shape.as_rapier_collider_shape(),
            grounded: false,
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

    pub fn build(self) -> CharacterController {
        self.character_controller
    }
}
