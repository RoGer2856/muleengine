#![allow(unstable_name_collisions)]
#![allow(
    clippy::comparison_chain,
    clippy::let_and_return,
    clippy::identity_op,
    clippy::needless_bool,
    clippy::collapsible_if
)]

extern crate miniz_oxide;
extern crate minizip_sys;

pub mod aabb;
pub mod app_loop_state;
pub mod application_runner;
pub mod asset_container;
pub mod asset_reader;
pub mod camera;
pub mod containers;
pub mod fps_counter;
pub mod heightmap;
pub mod image;
pub mod image_container;
pub mod mesh;
pub mod mesh_creator;
pub mod prelude;
pub mod renderer;
pub mod scene_container;
pub mod sendable_ptr;
pub mod service_container;
pub mod stopwatch;
pub mod sync;
pub mod system_container;
pub mod window_context;
