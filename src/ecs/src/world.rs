use mopa::Any;

use super::entity::Entities;
use super::component::Components;
use super::{Entity, System};
use std::any::TypeId;

/// The World type is responsible for managing the entities, components and systems. Entities
/// created through this type are sent to systems that accept their signature.
/// Systems are processed whenever `World::process` is called.
pub struct World {
    entities: Entities,
    components: Components,
    systems: Vec<Box<System>>,
    to_destroy: Vec<Entity>,
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

/// Systems cannot be added or removed to the world after it was created, to enforce this the
/// WorldBuilder object receives systems and is consumed to return an instace of a World.
/// # Example
/// ```
/// #[macro_use] extern crate luck_ecs;
///
/// fn main() {
///     use luck_ecs::{System, Signature, Entity, WorldBuilder};
///     use std::any::TypeId;
///
///     struct S1 {
///         entities: Vec<Entity>,
///     }
///     impl_signature!(S1, (u32, i32));
///     impl System for S1 {
///         fn has_entity(&self, entity: Entity) -> bool {
///             self.entities.iter().enumerate().find(|e| *e.1 == entity).is_some()
///         }
///         fn on_entity_added(&mut self, entity: Entity) {
///             self.entities.push(entity);
///         }
///         fn on_entity_removed(&mut self, entity: Entity) {
///             self.entities.retain(|&x| x != entity);
///         }
///     }
///
///     let w = WorldBuilder::new()
///                 .with_system(S1 { entities: vec![] })
///                 .build();
///
///     assert!(w.get_system::<S1>().is_some());
/// }
/// ```
pub struct WorldBuilder {
    systems: Vec<Box<System>>,
}

impl WorldBuilder {
    /// Constructs a new WorldBuilder which can be consumed to create a World object.
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        WorldBuilder { systems: Vec::new() }
    }

    /// Adds a system to the WorldBuilder, these systems will be permanent in the resulting
    /// World.
    pub fn with_system<T: System>(mut self, system: T) -> Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Consumes the WorldBuilder and return a new World.
    pub fn build(self) -> World {
        World {
            entities: Entities::new(),
            components: Components::new(),
            systems: self.systems,
            to_destroy: Vec::new(),
        }
    }

    /// Consumes the WorldBuilder and return a new World with memory pre-allocated for the Entity
    /// and Component vectors. Use this if you know how many Entities your scene will use.
    pub fn build_with_capacity(self, capacity: usize) -> World {
        World {
            entities: Entities::with_capacity(capacity),
            components: Components::with_capacity(capacity),
            systems: self.systems,
            to_destroy: Vec::new(),
        }
    }
}

fn match_entity_signature(system: &System, components: &Box<[TypeId]>) -> bool {
    let signature = system.signature();
    let mut count = 0;
    for s in &*signature {
        if components.contains(&s) {
            count = count + 1;
        }
    }

    count == signature.len()
}

impl World {
    /// Creates a new entity.
    pub fn create_entity(&mut self) -> Entity {
        self.entities.create_entity()
    }

    /// Destroy an enttiy. Memory is not released from entity destruction, the next entity
    /// created will reuse the id. Destroyed entities return false when checked through
    /// `World::is_valid`. Entities are only destroyed after the frame is over, calling
    /// `World::is_alive` right after `World::destroy_entity` will still return true.
    /// # Panics
    /// Panics if the entity is invalid or if it was already sent to be destroyed this frame. This
    /// is the only function that checks wether an entity is scheduled to be destroyed or not.
    pub fn destroy_entity(&mut self, entity: Entity) {
        assert!(self.entities.is_valid(entity) && !self.to_destroy.contains(&entity));

        self.to_destroy.push(entity);
    }

    /// Return the state of an entity, true if the entity is valid, false if the entity was
    /// destroyed or is invalid.
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn is_valid(&self, entity: Entity) -> bool {
        self.entities.is_valid(entity)
    }

    /// Adds a component to an entity. Only one component of each type can be added. If you add
    /// the same type twice, the new component will overwrite the old one. Don't forget to apply
    /// after you are done adding.
    /// # Panics
    /// Panics if the entity is invalid.
    pub fn add_component<T: Any>(&mut self, entity: Entity, component: T) -> &mut T {
        // TODO: instead of panicking, we could print a warning, we can just ignore invalid
        // entities anyway. Maybe a hard error in release mode.
        assert!(self.entities.is_valid(entity));
        self.components.add_component::<T>(entity.id() as usize, component)
    }

    /// Returns a reference to the component owned by the entity. Returns None if the entity
    /// doesn't have the component.
    /// # Panics
    /// Panics if the entity is invalid.
    pub fn get_component<T: Any>(&self, entity: Entity) -> Option<&T> {
        assert!(self.entities.is_valid(entity));
        self.components.get_component::<T>(entity.id() as usize)
    }

    /// Returns a multable reference to the component owned by the entity. Returns None if the
    /// entity doesn't have the component.
    /// # Panics
    /// Panics if the entity is invalid.
    pub fn get_component_mut<T: Any>(&mut self, entity: Entity) -> Option<&mut T> {
        assert!(self.entities.is_valid(entity));
        self.components.get_component_mut::<T>(entity.id() as usize)
    }

    /// Removes a component from an entity. Returns the removed component or None if the entity
    /// had no component of type T. Don't forget to apply after removing.
    /// # Panics
    /// Panics if the entity is invalid.
    pub fn remove_component<T: Any>(&mut self, entity: Entity) -> Option<T> {
        assert!(self.entities.is_valid(entity));
        self.components.remove_component::<T>(entity.id() as usize)
    }

    /// Removes every component from an entity. Don't forget to apply after removing.
    /// # Panics
    /// Panics if the entity is invalid
    pub fn remove_all_components(&mut self, entity: Entity) {
        assert!(self.entities.is_valid(entity));
        self.components.remove_all_components(entity.id() as usize)
    }

    /// Returns a reference to a system. Returns None if no system of type T can be found.
    pub fn get_system_mut<T: System>(&mut self) -> Option<&mut T> {
        self.systems.iter_mut().filter_map(|s| s.downcast_mut::<T>()).next()
    }

    /// Returns a multable reference to a system. Returns None if no system of type T can be found.
    pub fn get_system<T: System>(&self) -> Option<&T> {
        self.systems.iter().filter_map(|s| s.downcast_ref::<T>()).next()
    }

    /// Applies the changes made to an entity, refreshing the entity within the systems. This
    /// should be called after adding or removing components from an entity. Entity destruction
    /// doesn't have to be followed by an apply call.
    pub fn apply(&mut self, entity: Entity) {
        assert!(self.entities.is_valid(entity));

        let World { ref mut systems, ref mut components, .. } = *self;
        for system in systems.iter_mut() {
            if match_entity_signature(&**system,
                                      &components.generate_signature(entity.id() as usize)) {
                if !system.has_entity(entity) {
                    system.on_entity_added(entity);
                }
            } else if system.has_entity(entity) {
                system.on_entity_removed(entity);
            }
        }
    }

    /// Processes every system. The processing runs in two phases, a read only parallel phase
    /// and a read-write synchronized phase.
    pub fn process(&mut self) {
        use rayon::par_iter::*;

        let mut callbacks = Vec::with_capacity(self.systems.len());

        self.systems // TODO: make sure this is being run asynchronously
            .par_iter()
            .map(|s| s.process(self))
            .collect_into(&mut callbacks);

        for callback in &mut callbacks {
            (*callback)(self);
        }

        self.destroy_scheduled_entities();
    }

    fn destroy_scheduled_entities(&mut self) {
        let to_destroy = self.to_destroy.clone();
        for entity in to_destroy {
            self.remove_all_components(entity);
            self.apply(entity);
            self.entities.destroy_entity(entity);
        }
        self.to_destroy.clear();
    }
}

impl Drop for World {
    fn drop(&mut self) {
        for entity in &self.entities {
            self.to_destroy.push(entity);
        }

        self.destroy_scheduled_entities();
    }
}

#[cfg(test)]
mod test {
    use super::WorldBuilder;
    use super::super::{Signature, Entity, System, World};
    use std::ops::FnMut;
    use std::any::TypeId;
    use std;

    #[derive(Default, PartialEq, Debug)]
    struct PositionComponent(f32, f32, f32);
    #[derive(Default)]
    struct VelocityComponent(f32, f32, f32);

    #[derive(Default)]
    struct SpatialSystem {
        entities: Vec<Entity>,
        marker: bool,
    }
    impl_system!(SpatialSystem, (PositionComponent), {
        //std::thread::sleep(std::time::Duration::new(0, 500_000));
        //std::thread::sleep(std::time::Duration::new(10, 0));
        Box::new(move |w: &mut World|{
            if !w.get_system::<SpatialSystem>().unwrap().marker {
                // This system should always run first since it is inserted in the World before
                // the VelocitySystem.
                assert_eq!(w.get_system::<VelocitySystem>().unwrap().marker, false);
                w.get_system_mut::<SpatialSystem>().unwrap().marker = true;
            }
        })
    });
    impl Drop for SpatialSystem {
        fn drop(&mut self) {
            assert_eq!(self.entities.len(), 0);
        }
    }

    #[derive(Default)]
    struct VelocitySystem {
        entities: Vec<Entity>,
        marker: bool,
    }
    impl_system!(VelocitySystem, (PositionComponent, VelocityComponent), {
        //std::thread::sleep(std::time::Duration::new(10, 0));
        //std::thread::sleep(std::time::Duration::new(0, 250_000));

        let v1 = PositionComponent(0.0, 0.0, 0.0);

        Box::new(move |w: &mut World|{
            if !w.get_system::<VelocitySystem>().unwrap().marker {
                assert_eq!(w.get_system::<SpatialSystem>().unwrap().marker, true);
                w.get_system_mut::<VelocitySystem>().unwrap().marker = true;
                assert_eq!(v1, v1);
            }
        })
    });
    impl Drop for VelocitySystem {
        fn drop(&mut self) {
            assert_eq!(self.entities.len(), 0);
        }
    }

    #[test]
    fn creation() {
        let w = WorldBuilder::new()
                    .with_system(SpatialSystem::default())
                    .build();

        assert!(w.get_system::<SpatialSystem>().is_some());
        assert!(w.get_system::<VelocitySystem>().is_none());
        assert_eq!(w.systems.len(), 1);

        let w = WorldBuilder::new()
                    .with_system(SpatialSystem::default())
                    .with_system(VelocitySystem::default())
                    .build();

        assert!(w.get_system::<SpatialSystem>().is_some());
        assert!(w.get_system::<VelocitySystem>().is_some());
        assert_eq!(w.systems.len(), 2);
    }

    #[test]
    fn component_system_operations() {
        let mut w = WorldBuilder::new()
                        .with_system(SpatialSystem::default())
                        .with_system(VelocitySystem::default())
                        .build();

        let e1 = w.create_entity();
        w.add_component(e1, PositionComponent::default());
        w.add_component(e1, VelocityComponent::default());
        w.apply(e1);

        assert_eq!(w.get_system::<SpatialSystem>().unwrap().entities.len(), 1);
        assert_eq!(w.get_system::<SpatialSystem>().unwrap().has_entity(e1), true);
        assert_eq!(w.get_system::<VelocitySystem>().unwrap().entities.len(), 1);
        assert_eq!(w.get_system::<VelocitySystem>().unwrap().has_entity(e1), true);

        w.remove_component::<VelocityComponent>(e1);
        w.apply(e1);

        assert_eq!(w.get_system::<SpatialSystem>().unwrap().entities.len(), 1);
        assert_eq!(w.get_system::<SpatialSystem>().unwrap().has_entity(e1), true);
        assert_eq!(w.get_system::<VelocitySystem>().unwrap().entities.len(), 0);
        assert_eq!(w.get_system::<VelocitySystem>().unwrap().has_entity(e1), false);

        w.destroy_entity(e1);

        assert_eq!(w.get_system::<SpatialSystem>().unwrap().has_entity(e1), true);
        assert_eq!(w.get_system::<VelocitySystem>().unwrap().has_entity(e1), false);

        w.process();

        assert_eq!(w.get_system::<SpatialSystem>().unwrap().has_entity(e1), false);

        w.process();
    }

}
