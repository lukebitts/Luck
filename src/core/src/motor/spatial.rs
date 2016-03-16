use luck_math::{Vector3, Quaternion, Aabb};
use luck_ecs::{World, System, Entity, Signature, Components};
use std;
use std::any::TypeId;
use super::super::common::collections::DynamicTree;
use num::traits::{Zero, One};

#[derive(Clone)]
struct SpatialComponent {
    global_position: Vector3<f32>,
    local_position: Vector3<f32>,
    orientation: Quaternion,
    scale: Vector3<f32>,
    parent: Option<Entity>,
    children: Vec<Entity>,
    origin_aabb: Aabb,
    proxy: i32,
}

impl Default for SpatialComponent {
    fn default() -> Self {
        SpatialComponent {
            global_position: Vector3::zero(),
            local_position: Vector3::zero(),
            orientation: Quaternion::zero(),
            scale: Vector3::one(),
            parent: None,
            children: Vec::new(),
            origin_aabb: Aabb::new(Vector3::zero() - 1.0, Vector3::one()),
            proxy: -1,
        }
    }
}

impl SpatialComponent {
    pub fn new(local_position: Vector3<f32>, orientation: Quaternion, scale: Vector3<f32>, parent: Option<Entity>) -> Self{
        SpatialComponent {
            global_position: local_position,
            local_position: local_position,
            orientation: orientation,
            scale: scale,
            parent: parent,
            children: Vec::new(),
            origin_aabb: Aabb::new(Vector3::zero() - 1.0, Vector3::one()),
            proxy: -1,
        }
    }
    pub fn global_position(&self) -> Vector3<f32> {
        self.global_position
    }
    pub fn local_position(&self) -> Vector3<f32> {
        self.local_position
    }
    pub fn orientation(&self) -> Quaternion {
        self.orientation
    }
    pub fn scale(&self) -> Vector3<f32> {
        self.scale
    }
    pub fn children(&self) -> &[Entity] {
        //self.children.clone().into_boxed_slice()
        &self.children[..]
    }
}

#[derive(Default)]
struct SpatialSystem {
    entities: Vec<Entity>,
    tree: DynamicTree<Entity>
}

impl_signature!(SpatialSystem, (SpatialComponent));

impl SpatialSystem {
    pub fn set_parent(&mut self, child: Entity, parent: Option<Entity>, world: &mut World) {
        if let Some(parent) = parent {

        }
        else {
            let parent;
            {
                let comp = world.get_component_mut::<SpatialComponent>(child).unwrap();
                parent = comp.parent;
                comp.parent = None;

                comp.local_position = comp.global_position;
            }
            if let Some(parent) = parent {
                let parent_comp = world.get_component_mut::<SpatialComponent>(parent).unwrap();
                parent_comp.children.retain(|c| *c != child);
            }
        }
    }
    pub fn set_local_position(&mut self, entity: Entity, world: &mut World) {

    }
    pub fn set_global_position(&mut self, entity: Entity, world: &mut World) {

    }
}

impl System for SpatialSystem {
    fn has_entity(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }
    // TODO: Components parameter allow for the system to remove components, that should not be
    // possible since it might break other systems expectations.
    fn on_entity_added(&mut self, entity: Entity, components: &mut Components) {
        self.entities.push(entity);

        let mut spatial_component = components.get_component::<SpatialComponent>(entity.id() as usize).unwrap().clone();

        if let Some(parent) = spatial_component.parent {
            let parent_spatial = components.get_component_mut::<SpatialComponent>(parent.id() as usize).unwrap();
            spatial_component.global_position = parent_spatial.global_position + spatial_component.local_position;

            parent_spatial.children.push(entity);
        }

        let c = components.get_component_mut::<SpatialComponent>(entity.id() as usize).unwrap();
        c.global_position = spatial_component.global_position;

        let mut transformed_aabb = c.origin_aabb.clone();
        transformed_aabb.translate(c.global_position);

        c.proxy = self.tree.create_proxy(transformed_aabb, entity);

    }
    fn on_entity_removed(&mut self, entity: Entity, components: &mut Components, removed: &mut Components) {
        self.entities.retain(|&x| x != entity);

        let mut spatial_component = removed.get_component::<SpatialComponent>(entity.id() as usize).unwrap();

        if let Some(parent) = spatial_component.parent {
            let parent_global;
            {
                let parent_spatial = components.get_component_mut::<SpatialComponent>(parent.id() as usize).unwrap();

                for children in &spatial_component.children {
                    parent_spatial.children.push(*children);
                }

                parent_global = parent_spatial.global_position;
            }
            for children in &spatial_component.children {
                let child_spatial = components.get_component_mut::<SpatialComponent>(children.id() as usize).unwrap();
                child_spatial.parent = Some(parent);
                child_spatial.local_position = child_spatial.global_position - parent_global;
            }
        }

        self.tree.destroy_proxy(spatial_component.proxy);
    }
    /*fn process(&self, _: &World) -> Box<Fn(&mut World) + Send + Sync> {
        Box::new(move |mut w: &mut World|{

        })
    }*/
}

#[cfg(test)]
mod test {
    use super::{SpatialComponent, SpatialSystem};
    use luck_ecs::{WorldBuilder};
    use num::traits::One;
    use luck_math::{Vector3, Quaternion};

    #[test]
    fn children_test() {
        let mut w = WorldBuilder::new()
                    .with_system(SpatialSystem::default())
                    .build();

        let e1 = w.create_entity();
        w.add_component(e1, SpatialComponent::new(Vector3::new(10.0, 10.0, 10.0), Quaternion::default(), Vector3::one(), None));
        w.apply(e1);

        let e2 = w.create_entity();
        w.add_component(e2, SpatialComponent::new(Vector3::new(1.0, 1.0, 1.0), Quaternion::default(), Vector3::one(), Some(e1)));
        w.apply(e2);

        {
            let s = w.get_component::<SpatialComponent>(e2).unwrap();
            let p = w.get_component::<SpatialComponent>(e1).unwrap();
            assert_eq!(s.global_position, Vector3::new(11.0, 11.0, 11.0));
            assert!(p.children.contains(&e2));
        }

        let e3 = w.create_entity();
        w.add_component(e3, SpatialComponent::new(Vector3::new(1.0, 1.0, 1.0), Quaternion::default(), Vector3::one(), Some(e2)));
        w.apply(e3);

        {
            let s = w.get_component::<SpatialComponent>(e3).unwrap();
            let p = w.get_component::<SpatialComponent>(e2).unwrap();
            assert_eq!(s.global_position, Vector3::new(12.0, 12.0, 12.0));
            assert!(p.children.contains(&e3));
        }

        w.destroy_entity(e2);
        w.process();

        {
            let s = w.get_component::<SpatialComponent>(e3).unwrap();
            let p = w.get_component::<SpatialComponent>(e1).unwrap();
            assert_eq!(s.parent, Some(e1));
            assert_eq!(s.global_position, Vector3::new(12.0, 12.0, 12.0));
            assert_eq!(s.local_position, Vector3::new(2.0, 2.0, 2.0));
            assert!(p.children.contains(&e3));
        }



    }
}
