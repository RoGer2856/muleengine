use std::fmt::Debug;

pub mod command_channel;
mod prelude;

pub use async_worker_macros::async_worker_impl;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests2;

#[derive(Debug)]
pub struct AllWorkersDroppedError;

#[derive(Debug)]
pub struct AllClientsDroppedError;
