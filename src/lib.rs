#![allow(unstable_name_collisions)]
#![allow(
    clippy::comparison_chain,
    clippy::let_and_return,
    clippy::identity_op,
    clippy::needless_bool,
    clippy::collapsible_if
)]

pub mod app_loop_state;
pub mod application_runner;
pub mod async_systems_runner;
pub mod systems;
