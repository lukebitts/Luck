#![allow(unused_features)]
#![warn(missing_docs)]

// #![feature(test)]
// #![feature(fnbox)]

//! TODO: Fill the documentation

#[macro_use]
extern crate mopa;
extern crate rayon;

pub mod entity;
mod component;
#[macro_use]
pub mod system;
mod world;

pub use entity::Entity;
pub use component::Components;
pub use system::{System, Signature};
pub use world::{World, WorldBuilder};
