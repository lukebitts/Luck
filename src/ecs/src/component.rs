//! A module for the `Components` type. Through a `Components` you can add and remove
//! any type that implements `Any` and has no non-static references.
//! Should be used through the `World` and not directly.
extern crate anymap;

use self::anymap::AnyMap;
use std::any::{Any, TypeId};

/// This type holds a `Vec<AnyMap>`. Entities are identified by their id (the 'key' of the
/// vector) and AnyMap can hold one of each component type. An entity can only have either
/// 0 or 1 component for a given component type. If you have entities 1 and 500 alive the
/// vector will keep 500 `AnyMap`'s in memory. Even if you destroy every entity the memory
/// of the components won't be freed. There's no way to "drain" the memory due to the
/// way entity handles work.
#[derive(Debug)]
pub struct Components {
    components: Vec<AnyMap>,
    signatures: Vec<Box<[TypeId]>>,
}

impl Components {
    /// Constructs a new instance of `Components`. The internal vector is empty and will only
    /// allocate when a component is added.
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        Components {
            components: Vec::new(),
            signatures: Vec::new(),
        }
    }

    /// Constructs a new instance of `Components`. The internal vector is initialized with the
    /// specified capacity.
    #[allow(unknown_lints)]
    #[allow(inline_always)]
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Components {
            components: Vec::with_capacity(capacity),
            signatures: Vec::new(),
        }
    }

    /// Returns a list with every component associated with the `index`.
    pub fn generate_signature(&mut self, index: usize) -> Box<[TypeId]> {
        self.signatures.get(index).cloned().unwrap_or_default()
    }

    /// Adds the `component` to the internal component list associated with the number
    /// `index`.
    pub fn add_component<T: Any>(&mut self, index: usize, component: T) -> &mut T {
        while self.components.len() <= index {
            self.components.push(AnyMap::new());
            self.signatures.push(Box::new([]));
        }

        //TODO: this should not replace the component, it should ignore it.
        match self.components[index].insert(component) {
            Some(_) => (),
            None => {
                let mut signature = Vec::new();
                signature.extend_from_slice(&*self.signatures[index]);
                signature.push(TypeId::of::<T>());
                self.signatures[index] = signature.into_boxed_slice();
            }
        }

        self.get_component_mut::<T>(index)
            .expect("Component we just added was not found. This should never happen")
    }

    /// If there is a component of type T associated with the number `index`, a reference to this
    /// component is returned. If index is out of bounds or the number is not associated with the
    /// component type, None is returned.
    pub fn get_component<T: Any>(&self, index: usize) -> Option<&T> {
        if let Some(map) = self.components.get(index) {
            map.get::<T>()
        } else {
            None
        }
    }

    /// If there is a component of type T associated with the number `index`, a mutable reference
    /// to this component is returned. If index is out of bounds or the number is not associated
    /// with the component type, None is returned.
    pub fn get_component_mut<T: Any>(&mut self, index: usize) -> Option<&mut T> {
        if let Some(map) = self.components.get_mut(index) {
            map.get_mut::<T>()
        } else {
            None
        }
    }

    /// Removes the component `T` associated with the number `index` and returns it.
    pub fn remove_component<T: Any>(&mut self, index: usize) -> Option<T> {
        if let Some(map) = self.components.get_mut(index) {
            let mut signature = Vec::new();
            signature.extend_from_slice(&*self.signatures[index]);
            signature.retain(|x| *x != TypeId::of::<T>());
            self.signatures[index] = signature.into_boxed_slice();

            map.remove::<T>()
        } else {
            None
        }
    }

    /// Removes every component associated with the `index`.
    pub fn remove_all_components(&mut self, index: usize) -> Option<Components> {
        use std::mem::replace;

        let cloned_signatures = self.signatures.clone();
        if let Some(ret) = self.components.get_mut(index).map(|map| {
            let mut vec = Vec::with_capacity(index + 1);
            while vec.len() <= index { vec.push(AnyMap::new()); }
            vec[index] = replace(map, AnyMap::new());

            Components {
                components: vec,
                signatures: cloned_signatures
            }
            //*map = AnyMap::new()
        }) {
            self.signatures[index] = Box::new([]);
            Some(ret)
        }
        else {
            None
        }
    }

    /// Resets the Components object.
    pub fn clear(&mut self) {
        *self = Components::new();
    }

    /// Merges two `Components` objects. The process is basically components in `other` are
    /// added to `target`, if `target` already has components associated with an index, they
    /// are overwritten. `other` is consumed in the process.
    pub fn merge(target: &mut Self, mut other: Self) {
        use std;
        for (i, map) in other.components.iter_mut().enumerate() {

            while target.components.len() <= i {
                target.components.push(AnyMap::new());
                target.signatures.push(Box::new([]));
            }

            if !map.is_empty() {
                target.components[i] = std::mem::replace(map, AnyMap::new());
                target.signatures[i] = std::mem::replace(&mut other.signatures[i], Box::new([]));
            }

        }
    }
}

#[cfg(test)]
mod test {
    use super::Components;

    #[derive(Debug, Eq, PartialEq)]
    struct FooComponent(u32);

    #[test]
    fn with_reference() {
        static INT_REF: &'static i32 = &15;

        #[derive(Debug, Eq, PartialEq)]
        struct RefHolder<'a> {
            r: &'a i32,
        }

        let mut comp_list = Components::new();
        comp_list.add_component(0usize, RefHolder { r: INT_REF });

        assert_eq!(comp_list.get_component_mut::<RefHolder>(0usize).unwrap().r, &15);
    }

    #[test]
    fn addition_and_recovery() {
        let mut comp_list = Components::new();
        for index in 0usize..100_000usize {
            assert_eq!(*comp_list.add_component(index, FooComponent(0u32)), FooComponent(0u32));
            assert_eq!(*comp_list.get_component::<FooComponent>(index).unwrap(),
                       FooComponent(0u32));
            assert_eq!(*comp_list.get_component_mut::<FooComponent>(index).unwrap(),
                       FooComponent(0u32));
        }
    }

    #[test]
    fn removal() {
        let mut comp_list = Components::new();
        let index = 0usize;

        assert_eq!(*comp_list.add_component(index, FooComponent(0u32)), FooComponent(0u32));
        assert_eq!(*comp_list.add_component(index, FooComponent(1u32)), FooComponent(1u32));

        assert_eq!(*comp_list.get_component::<FooComponent>(index).unwrap(), FooComponent(1u32));
        assert_eq!(comp_list.remove_component::<FooComponent>(index).unwrap(), FooComponent(1u32));
        assert_eq!(comp_list.get_component::<FooComponent>(index).is_none(), true);
        assert_eq!(comp_list.remove_component::<FooComponent>(index).is_none(), true);
    }

    #[test]
    fn merging_and_clearing() {

        let mut comp_list1 = Components::new();
        let mut comp_list2 = Components::new();
        let index1 = 0usize;
        let index2 = 1usize;

        comp_list1.add_component(index1, FooComponent(0u32));
        comp_list2.add_component(index2, FooComponent(0u32));

        Components::merge(&mut comp_list1, comp_list2);

        assert_eq!(comp_list1.components.len(), 2);

        comp_list1.clear();
        assert_eq!(comp_list1.components.len(), 0);
    }
}
