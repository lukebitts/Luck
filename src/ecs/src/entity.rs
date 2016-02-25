//! A module for two types, `Entity`and `Entities`. `Entity` represents an entity inside an
//! `Entities` object which is responsible for managing an entity lifetime. The `Entities` type
//! should be used through the `World` and not directly.

use std::iter;

/// EntityId is a type that changes according to the pointer size of the target machines.
/// It is supported `u64` for x64 machines and `u32` for x86 machines. Machines with
/// different sizes might not work.
pub type EntityId = u64;
type EntityKey = u64;

/// A type used to represent an entity. Objects of this type can be copied and `Entities::is_alive`
/// is guaranteed to return false if the entity was destroyed, even taking in account id reuse.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Entity {
    id: EntityId,
    key: EntityKey,
}

impl Entity {
    /// Returns the id of the entity in the Entities object (or the World). You can't
    /// differentiate dead or alive entities just by their id.
    pub fn id(&self) -> EntityId {
        self.id
    }
}

/// An object to hold entities and their ids. Entities are stored sequentially and
/// when an entity is destroyed, it's id is reused and old instances of Entity objects that pointed
/// to the destroyed entity are considered dead.
pub struct Entities {
    free_entity_ids: Vec<EntityId>,
    entities: Vec<EntityKey>,
}

impl Entities {
    // Generates a new entity id and key either by reusing old ones or creating new ones.
    fn generate_entity_id(&mut self) -> (EntityId, EntityKey) {
        let free_id = self.free_entity_ids.pop();

        match free_id {
            None => {
                self.entities.push(1);
                (self.entities.len() as EntityId - 1, 1)
            }
            Some(free_id) => {
                let key = unsafe { self.entities.get_unchecked(free_id as usize) };
                (free_id, *key)
            }
        }
    }

    /// Constructs a new instance of `Entities`. The internal vectors are empty and will only
    /// allocate when an entity is created.
    /// # Examples
    /// ```
    /// use luck_ecs::entity::Entities;
    /// let mut entities = Entities::new();
    /// ```
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        Entities {
            free_entity_ids: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Constructs a new instance of `Entities`. The internal vectors are initialized with the
    /// specified capacity.
    /// # Examples
    /// ```
    /// use luck_ecs::entity::Entities;
    /// let mut entities: Entities = Entities::with_capacity(10);
    /// //The resulting Entities object will only allocate after the 11th entity is created.
    /// ```
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Entities {
            free_entity_ids: Vec::with_capacity(capacity),
            entities: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new entity and return it's identification.
    /// # Examples
    /// ```
    /// use luck_ecs::entity::Entities;
    /// let mut entities: Entities = Entities::with_capacity(1);
    /// let entity = entities.create_entity();
    /// ```
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn create_entity(&mut self) -> Entity {
        let (id, key) = self.generate_entity_id();
        Entity { id: id, key: key }
    }

    /// Marks an entity as dead. The entity object is still in a valid state but call to
    /// `Entity::is_valid` will return false. Dead entities are ignored by the function.
    /// # Examples
    /// ```
    /// use luck_ecs::entity::Entities;
    /// let mut entities: Entities = Entities::with_capacity(1);
    /// let entity = entities.create_entity();
    /// entities.destroy_entity(entity);
    /// assert!(!entities.is_valid(entity));
    /// ```
    pub fn destroy_entity(&mut self, entity: Entity) {
        if self.is_valid(entity) {
            self.free_entity_ids.push(entity.id);
            self.entities[entity.id as usize] = self.entities[entity.id as usize] + 1;
        }
    }

    /// Returns the state of an entity. Entities created through an `Entities` object will return
    /// true. If they are destroyed or the `Entity` is invalid it will return false.
    /// # Examples
    /// ```
    /// use luck_ecs::entity::Entities;
    /// let mut entities: Entities = Entities::with_capacity(1);
    /// let entity = entities.create_entity();
    /// entities.destroy_entity(entity);
    /// assert!(!entities.is_valid(entity));
    /// ```
    pub fn is_valid(&self, entity: Entity) -> bool {
        if let Some(key) = self.entities.get(entity.id as usize) {
            *key == entity.key
        } else {
            false
        }
    }
}

impl iter::IntoIterator for Entities {
    type Item = Entity;
    type IntoIter = EntitiesIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        EntitiesIntoIterator {
            entities: self,
            index: 0,
        }
    }
}

impl<'a> iter::IntoIterator for &'a Entities {
    type Item = Entity;
    type IntoIter = EntitiesIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EntitiesIterator {
            entities: self,
            index: 0,
        }
    }
}

/// An iterator that moves out the Entities object. Returns only entities that are valid.
pub struct EntitiesIntoIterator {
    entities: Entities,
    index: usize,
}

impl iter::Iterator for EntitiesIntoIterator {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(key) = self.entities.entities.get(self.index) {
                self.index = self.index + 1;
                if !self.entities.free_entity_ids.contains(&((self.index - 1) as EntityId)) {
                    return Some(Entity {
                        id: (self.index - 1) as EntityId,
                        key: *key,
                    });
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }
    }
}

/// An iterator that does not move the Entities object. Returns only entities that are valid.
pub struct EntitiesIterator<'a> {
    entities: &'a Entities,
    index: usize,
}

impl<'a> iter::Iterator for EntitiesIterator<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(key) = self.entities.entities.get(self.index) {
                self.index = self.index + 1;
                if !self.entities.free_entity_ids.contains(&((self.index - 1) as EntityId)) {
                    return Some(Entity {
                        id: (self.index - 1) as EntityId,
                        key: *key,
                    });
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }
    }
}


#[cfg(test)]
mod test {
    // extern crate test;
    extern crate rand;
    use self::rand::{Rng, thread_rng};
    // use self::test::Bencher;
    use super::{Entity, Entities, EntityId};

    // Benchmark to test time time it takes to create a million entities, with level 3
    // optimizations the average time is 2,545,401 ns
    // #[bench]
    // fn creation_time(b: &mut Bencher) {
    //     fn c() {
    //         let mut entities: Entities = Entities::with_capacity(1_000_000usize);
    //         for _ in 0..1_000_000 {
    //             let _: Entity = entities.create_entity();
    //         }
    //     }
    //     b.iter(c);
    // }

    // Tests the creation of 500.000 entities and the generation of their id's
    #[test]
    fn creation() {
        let mut entities: Entities = Entities::with_capacity(500_000usize);

        let base = EntityId::max_value() - 500_000;
        for i in base..EntityId::max_value() {
            let e: Entity = entities.create_entity();
            assert_eq!(((e.id + base), e.key), (i, 1));
        }
    }

    // Tests the destruction and recreation of the entities and the reuse of deleted entity id's
    #[test]
    fn destruction_recreation() {
        let sample = 500_000usize;
        let mut entities: Entities = Entities::with_capacity(sample);

        let mut entity_list = Vec::with_capacity(sample);
        for i in 0..sample {
            let e: Entity = entities.create_entity();
            assert_eq!((e.id, e.key), (i as EntityId, 1));
            entity_list.push(e);
        }

        thread_rng().shuffle(&mut entity_list[..]);

        for e in entity_list {
            entities.destroy_entity(e);
            assert_eq!(entities.is_valid(e), false);
            let ne: Entity = entities.create_entity();
            assert_eq!((ne.id, ne.key), (e.id, e.key + 1));
        }
    }

    #[test]
    // Tests the iterator logic
    fn iteration() {
        let mut entities: Entities = Entities::new();

        let ent_list = [entities.create_entity(),
                        entities.create_entity(),
                        entities.create_entity(),
                        entities.create_entity()];

        for (i, e) in (&entities).into_iter().enumerate() {
            assert_eq!(e, ent_list[i]);
        }

        entities.destroy_entity(ent_list[2]);

        let mut iter = (&entities).into_iter();
        assert_eq!(iter.next(), Some(ent_list[0]));
        assert_eq!(iter.next(), Some(ent_list[1]));
        assert_eq!(iter.next(), Some(ent_list[3]));
        assert_eq!(iter.next(), None);

    }

    // Test to check if EntityId is smaller or equal to usize, since vectors use usize as key and
    // EntityId is used as the key of the vector.
    #[test]
    #[allow(unknown_lints)]
    #[allow(unused_comparisons)]
    #[allow(absurd_extreme_comparisons)]
    fn type_size() {
        let max_id = EntityId::max_value() as u64;
        let max_usize = usize::max_value() as u64;

        if max_id > max_usize || EntityId::min_value() < 0 {
            panic!("Type must be contained by usize")
        }
    }
}
