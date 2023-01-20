use vek::{Vec2, Vec3};

use muleengine::messaging::mpmc;

#[derive(Clone)]
pub struct SpectatorCameraInput {
    pub(super) moving_event_receiver: mpmc::Receiver<Vec3<f32>>,
    pub(super) turning_event_receiver: mpmc::Receiver<Vec2<f32>>,
}
