//! System is the trait the must be implemented by every system.
//! # Example
//! ```
//! #![feature(fnbox)]
//! use luck_ecs::{Entity, System, Signature, World};
//! use std::any::TypeId;
//! use std::boxed::FnBox;
//!
//! struct S1 {
//!     entities: Vec<Entity>
//! }
//!
//!
//! //You can implement the signature yourself or use the `impl_signature!` macro.
//! impl Signature for S1 {
//!     fn signature(&self) -> Box<[TypeId]> {
//!         Box::new([
//!             TypeId::of::<u32>(),
//!             TypeId::of::<i32>()
//!         ])
//!     }
//! }
//!
//! impl System for S1 {
//!     fn has_entity(&self, entity: Entity) -> bool {
//!         self.entities.iter().enumerate().find(|e| *e.1 == entity).is_some()
//!     }
//!     fn on_entity_added(&mut self, entity: Entity) {
//!         self.entities.push(entity);
//!     }
//!     fn on_entity_removed(&mut self, entity: Entity) {
//!         self.entities.retain(|&x| x != entity);
//!     }
//!     fn process(&self, _: &World) -> Box<FnBox(&mut World) + Send + Sync> {
//!         //[...]
//!         //Read only operations, like finding which entities need processing.
//!         Box::new(move |w: &mut World|{
//!             //[...]
//!             //Operations that mutate the world, you can access the system state through
//!             //w.get_system::<S1>()
//!         })
//!     }
//! }
//! ```

use std::any::TypeId;
use std::boxed::FnBox;

use super::Entity;
use super::World;
use mopa;

/// A trait that describes which components the system should process. It is split from the
/// System trait to allow it to be implemented through the impl_signature macro.
pub trait Signature : mopa::Any + Send + Sync  {
    /// Should return the components this system expects to process.
    fn signature(&self) -> Box<[TypeId]>;
}

/// A trait that every System struct should implement.
pub trait System : Signature {
    // TODO: Add a on_drop event? Implementing Drop for a system is useless since it is only
    // called after the World already cleaned it.

    /// Should return true if an entity add event has been received by this System.
    fn has_entity(&self, entity: Entity) -> bool;

    /// This event is fired everytime the signature of an entity matches the signature of the
    /// system and the system has not received this entity yet (checked through has_entity).
    fn on_entity_added(&mut self, entity: Entity);

    /// This event is fired everytime the signature of an entity doesn't match the signature of the
    /// system the system has a reference to this entity (checked through has_entity).
    fn on_entity_removed(&mut self, entity: Entity);

    /// This event is fired every frame. Only read only operations can be done during the proccess
    /// itself since this step is run concurrently. Multable changes have to be done inside the
    /// returning function witch will be run in order depending on the orther the systems were
    /// added to the World.
    fn process(&self, _: &World) -> Box<FnBox(&mut World) + Send + Sync> {
        fn ret(_: &mut World) {}
        Box::new(ret)
    }
}

mopafy!(System);

/// A macro to make it easier to implement the Signature trait.
/// # Example
/// ```
/// #[macro_use] extern crate luck_ecs;
///
/// fn main() {
///     use luck_ecs::{Entity, System, Signature, World};
///     use std::any::TypeId;
///
///     struct S1 {
///         entities: Vec<Entity>
///     }
///
///     impl_signature!(S1, (u32, i32));
/// }
/// ```
#[macro_export]
macro_rules! impl_signature {
    ( $name:ty , ( $( $mask:path ),+ ) ) => {
        impl<'a> Signature for $name {
            fn signature(&self) -> Box<[TypeId]> {
                Box::new([ $(std::any::TypeId::of::<$mask>()),+ ])
            }
        }
    }
}

macro_rules! impl_system {
    ( $name:ty , ( $( $mask:path ),+ ) , $process:block ) => {
        impl_signature!($name, ( $($mask),+ ) );
        impl<'a> System for $name {
            fn has_entity(&self, entity: Entity) -> bool {
                self.entities.iter().enumerate().find(|e| *e.1 == entity).is_some()
            }
            fn on_entity_added(&mut self, entity: Entity) {
                self.entities.push(entity);
            }
            fn on_entity_removed(&mut self, entity: Entity) {
                self.entities.retain(|&x| x != entity);
            }
            fn process(&self, _: &World) -> Box<FnBox(&mut World) + Send + Sync> {
                $process
            }
        }
    };

    ( $name:ty , ( $( $mask:path ),+ ) ) => {
        impl_system!($name, ( $($mask),+ ), {
            fn ret(_: &mut World) {}
            Box::new(ret)
        });
    };
}
