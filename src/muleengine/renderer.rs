use std::sync::{mpsc, Arc};

use parking_lot::RwLock;
use vek::{Transform, Vec2};

use super::{camera::Camera, drawable_object::DrawableObject};

pub enum Command {
    AddDrawableObject {
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
        transform: Transform<f32, f32, f32>,
    },
    SetCamera {
        camera: Camera,
    },
    SetWindowDimensions {
        dimensions: Vec2<usize>,
    },
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;

pub fn command_channel() -> (CommandSender, CommandReceiver) {
    mpsc::channel()
}
