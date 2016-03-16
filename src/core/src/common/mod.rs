#![allow(missing_docs)]
#![allow(dead_code)] // FIXME
//! A module that defines common types used throught the engine.

mod material;
mod mesh;
mod vertex;
mod resource;
pub mod collections;

pub use self::vertex::*;
pub use self::mesh::*;
pub use self::material::*;
pub use self::resource::*;
