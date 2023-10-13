use vek::{Vec2, Vec3};

use muleengine::sync::mpmc;

#[derive(Clone)]
pub enum VelocityChangeEvent {
    Accelerate,
    Decelerate,
}

#[derive(Clone)]
pub(super) struct FpsCameraInput {
    pub(super) velocity_change_event_receiver: mpmc::Receiver<VelocityChangeEvent>,
    pub(super) moving_event_receiver: mpmc::Receiver<Vec3<f32>>,
    pub(super) turning_event_receiver: mpmc::Receiver<Vec2<f32>>,
}
