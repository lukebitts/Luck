#![warn(missing_docs)]
#![warn(unused)]

#![feature(plugin)]

#![plugin(clippy)]

//! TODO: Fill the documentation

extern crate glm;
extern crate num;

pub mod aabb;
mod quaternion;
mod extensions;

pub use glm::*;
pub use aabb::Aabb;
pub use quaternion::*;
pub use extensions::*;
