#![allow(unstable_name_collisions)]
#![allow(
    clippy::comparison_chain,
    clippy::let_and_return,
    clippy::identity_op,
    clippy::needless_bool,
    clippy::collapsible_if
)]

pub mod aabb;
pub mod asset_container;
pub mod asset_reader;
pub mod camera;
pub mod containers;
pub mod heightmap;
pub mod image;
pub mod image_container;
pub mod mesh;
pub mod mesh_creator;
pub mod messaging;
pub mod prelude;
pub mod renderer;
mod result_option_inspect;
pub mod scene_container;
pub mod sendable_ptr;
pub mod service_container;
pub mod system_container;
pub mod window_context;
