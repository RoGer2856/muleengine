use vek::{Vec2, Vec3};

use muleengine::bytifex_utils::sync::broadcast;

#[derive(Clone)]
pub enum VelocityChangeEvent {
    Accelerate,
    Decelerate,
}

#[derive(Clone)]
pub(super) struct FlyingSpectatorCameraInput {
    pub(super) velocity_change_event_receiver: broadcast::Receiver<VelocityChangeEvent>,
    pub(super) moving_event_receiver: broadcast::Receiver<Vec3<f32>>,
    pub(super) turning_event_receiver: broadcast::Receiver<Vec2<f32>>,
}
