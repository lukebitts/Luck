#![allow(missing_docs)]
/*
    This is a port of the b2DynamicTree from Box2D, which in turn is inspired by
    Nathanael Presson's btDbvt. The changes include the removal of Box2D dependecies
    and the addition of another dimension.
    /// TODO: Find box2d's license.
*/

use luck_math;
use luck_math::{Vector3, Aabb};
use std::cmp::max;

type Vec3 = Vector3<f32>;

#[derive(Clone, Debug)]
struct TreeNode<UserData: Copy>
{
    aabb: Aabb,
    user_data: Option<UserData>,
    parent: i32,
    next: i32,
    child1: i32,
    child2: i32,
    height: i32
}

#[inline(always)]
fn null_node() -> i32 {
    -1
}

impl<UserData: Copy> TreeNode<UserData> {
    fn new() -> Self {
        TreeNode {
            aabb: Aabb::new(Vector3::new(0f32, 0f32, 0f32), Vector3::new(0f32, 0f32, 0f32)),
            user_data: None,
            parent: null_node(),
            next: null_node(),
            child1: null_node(),
            child2: null_node(),
            height: 0
        }
    }
    fn is_leaf(&self) -> bool {
        self.child1 == null_node()
    }
}

#[derive(Debug)]
pub struct DynamicTree<UserData: Copy>
{
    root: i32,
    nodes: Vec<TreeNode<UserData>>,
    free_list: i32,
    path: u32,
}

impl<UserData: Copy> Default for DynamicTree<UserData> {
    fn default() -> DynamicTree<UserData> {
        DynamicTree::new()
    }
}


impl<UserData: Copy> DynamicTree<UserData> {
    fn new() -> Self {
        let node_capacity = 16i32;
        let mut dtree = DynamicTree {
            root: null_node(),
            nodes: Vec::with_capacity(node_capacity as usize),
            free_list: 0,
            path: 0,
        };

        for i in 0..node_capacity {
            dtree.nodes.push(TreeNode::new());
            let mut node = &mut dtree.nodes[i as usize];
            node.next = i + 1;
            node.height = -1;
        }

        {
            let mut last_node = dtree.nodes.last_mut().unwrap();
            last_node.next = null_node();
            last_node.height = -1;
        }

        dtree
    }

    fn allocate_node(&mut self) -> i32 {
        if self.free_list == null_node() {
            let mut node = TreeNode::new();
            node.height = -1;

            self.nodes.push(node);
            self.free_list = (self.nodes.len() - 1) as i32;
        }

        let node_id = self.free_list;
        let ref node = self.nodes[node_id as usize];
        self.free_list = node.next;

        node_id
    }

    fn free_node(&mut self, node_id : i32) {
        assert!(0 <= node_id && node_id < self.nodes.len() as i32);

        let mut node = self.nodes.get_mut(node_id as usize).unwrap();
        node.next = self.free_list;
        node.height = -1;
        self.free_list = node_id;
    }

    pub fn create_proxy(&mut self, aabb: Aabb, user_data: UserData) -> i32 {
        let mut aabb = aabb;
        let proxy_id = self.allocate_node();
        let aabb_extension = 0.1f32;

        {
            let mut node = self.nodes.get_mut(proxy_id as usize).unwrap();

            Self::extend_aabb(&mut aabb, aabb_extension);

            node.aabb = aabb;
            node.user_data = Some(user_data);
            node.height = 0;
        }
        self.insert_leaf(proxy_id);

        proxy_id
    }

    pub fn destroy_proxy(&mut self, proxy_id: i32) {
        assert!(0 <= proxy_id && proxy_id < self.nodes.len() as i32);
        assert!(self.nodes[proxy_id as usize].is_leaf());

        self.remove_leaf(proxy_id);
        self.free_node(proxy_id);
    }

    pub fn move_proxy(&mut self, proxy_id: i32, aabb: Aabb, displacement: Vector3<f32>) -> bool {
        assert!(0 <= proxy_id && proxy_id < self.nodes.len() as i32);
        assert!(self.nodes[proxy_id as usize].is_leaf());

        if self.nodes[proxy_id as usize].aabb.contains(aabb) {
            return false
        }

        self.remove_leaf(proxy_id);

        let aabb_extension : f32 = 0.1;
        let aabb_multiplier : f32 = 2.0;

        let mut aabb = aabb;
        Self::extend_aabb(&mut aabb, aabb_extension);

        let d = displacement * aabb_multiplier;

        if d.x < 0.0 {
            aabb.min.x += d.x;
        }
        else {
            aabb.max.x += d.x;
        }

        if d.y < 0.0 {
            aabb.min.y += d.y;
        }
        else {
            aabb.max.y += d.y;
        }

        if d.z < 0.0 {
            aabb.min.z += d.z;
        }
        else {
            aabb.max.z += d.z;
        }

        self.nodes.get_mut(proxy_id as usize).unwrap().aabb = aabb;
        self.insert_leaf(proxy_id);

        true
    }

    fn insert_leaf(&mut self, leaf: i32) {
        if self.root == null_node() {
            self.root = leaf;
            self.nodes[self.root as usize].parent = null_node();
            return
        }

        let leaf_aabb = self.nodes[leaf as usize].aabb;
        let mut index = self.root;

        while self.nodes[index as usize].is_leaf() == false {
            let child1 = self.nodes[index as usize].child1;
            let child2 = self.nodes[index as usize].child2;

            let area = self.nodes[index as usize].aabb.perimeter();

            let mut combined_aabb = Aabb::default();
            combined_aabb.combine(self.nodes[index as usize].aabb, leaf_aabb);
            let combined_area = combined_aabb.perimeter();

            let cost = 2.0 * combined_area;
            let inheritance_cost = 2.0 * (combined_area - area);

            let cost1 : f32;

            if self.nodes[child1 as usize].is_leaf() {
                let mut aabb = Aabb::default();
                aabb.combine(leaf_aabb, self.nodes[child1 as usize].aabb);
                cost1 = aabb.perimeter() + inheritance_cost;
            }
            else {
                let mut aabb = Aabb::default();
                aabb.combine(leaf_aabb, self.nodes[child1 as usize].aabb);
                let old_area = self.nodes[child1 as usize].aabb.perimeter();
                let new_area = aabb.perimeter();
                cost1 = (new_area - old_area) + inheritance_cost;
            }

            let cost2 : f32;
            if self.nodes[child2 as usize].is_leaf() {
                let mut aabb = Aabb::default();
                aabb.combine(leaf_aabb, self.nodes[child2 as usize].aabb);
                cost2 = aabb.perimeter() + inheritance_cost;
            }
            else {
                let mut aabb = Aabb::default();
                aabb.combine(leaf_aabb, self.nodes[child2 as usize].aabb);
                let old_area = self.nodes[child2 as usize].aabb.perimeter();
                let new_area = aabb.perimeter();
                cost2 = (new_area - old_area) + inheritance_cost;
            }

            if cost < cost1 && cost < cost2 {
                break
            }
            if cost1 < cost2 {
                index = child1;
            }
            else {
                index = child2;
            }
        }

        let sibling = index;

        let old_parent = self.nodes[sibling as usize].parent;
        let new_parent = self.allocate_node();
        self.nodes[new_parent as usize].parent = old_parent;
        self.nodes[new_parent as usize].user_data = None;

        let _temp_aabb = self.nodes[sibling as usize].aabb;
        self.nodes[new_parent as usize].aabb.combine(leaf_aabb, _temp_aabb);
        self.nodes[new_parent as usize].height = self.nodes[sibling as usize].height + 1;

        if old_parent != null_node() {
            if self.nodes[old_parent as usize].child1 == sibling {
                self.nodes[old_parent as usize].child1 = new_parent;
            }
            else {
                self.nodes[old_parent as usize].child2 = new_parent;
            }

            self.nodes[new_parent as usize].child1 = sibling;
            self.nodes[new_parent as usize].child2 = leaf;
            self.nodes[sibling as usize].parent = new_parent;
            self.nodes[leaf as usize].parent = new_parent;
        }
        else {
            self.nodes[new_parent as usize].child1 = sibling;
            self.nodes[new_parent as usize].child2 = leaf;
            self.nodes[sibling as usize].parent = new_parent;
            self.nodes[leaf as usize].parent = new_parent;
            self.root = new_parent;
        }

        index = self.nodes[leaf as usize].parent;
        while index != null_node() {
            index = self.balance(index);

            let child1 = self.nodes[index as usize].child1;
            let child2 = self.nodes[index as usize].child2;

            assert!(child1 != null_node());
            assert!(child2 != null_node());

            self.nodes[index as usize].height = 1 + max(self.nodes[child1 as usize].height,self.nodes[child2 as usize].height);

            let _temp_aabb1 = self.nodes[child1 as usize].aabb;
            let _temp_aabb2 = self.nodes[child2 as usize].aabb;
            self.nodes[index as usize].aabb.combine(_temp_aabb1, _temp_aabb2);

            index = self.nodes[index as usize].parent;
        }
    }

    fn remove_leaf(&mut self, leaf: i32) {
        if leaf == self.root {
            self.root = null_node();
            return;
        }

        let parent = self.nodes[leaf as usize].parent;
        let grand_parent = self.nodes[parent as usize].parent;
        let sibling : i32;

        if self.nodes[parent as usize].child1 == leaf {
            sibling = self.nodes[parent as usize].child2;
        }
        else {
            sibling = self.nodes[parent as usize].child1;
        }

        if grand_parent != null_node() {
            if self.nodes[grand_parent as usize].child1 == parent {
                self.nodes[grand_parent as usize].child1 = sibling;
            }
            else {
                self.nodes[grand_parent as usize].child2 = sibling;
            }
            self.nodes[sibling as usize].parent = grand_parent;
            self.free_node(parent);

            let mut index = grand_parent;
            while index != null_node() {
                index = self.balance(index);

                let child1 = self.nodes[index as usize].child1;
                let child2 = self.nodes[index as usize].child2;

                let (temp_aabb1, temp_aabb2) = (self.nodes[child1 as usize].aabb, self.nodes[child2 as usize].aabb);
                self.nodes[index as usize].aabb.combine(temp_aabb1, temp_aabb2);
                self.nodes[index as usize].height = 1 + max(self.nodes[child1 as usize].height, self.nodes[child2 as usize].height);

                index = self.nodes[index as usize].parent;
            }
        }
        else {
            self.root = sibling;
            self.nodes[sibling as usize].parent = null_node();
            self.free_node(parent);
        }
    }

    fn balance(&mut self, i_a: i32) -> i32 {
        macro_rules! b {
            ( $x:expr ) => {
                self.nodes[$x as usize]
            }
        }

        assert!(i_a != null_node());

        let nodes_len = self.nodes.len() as i32;

        //let ref mut a = b![i_a as usize];
        if b![i_a].is_leaf() || b![i_a].height < 2 {
            return i_a
        }

        let i_b = b![i_a].child1;
        let i_c = b![i_a].child2;

        assert!(0 <= i_b && i_b < nodes_len);
        assert!(0 <= i_c && i_c < nodes_len);

        //let ref mut b = self.nodes[i_b as usize];
        //let ref mut c = self.nodes[i_c as usize];

        let balance = b![i_c].height - b![i_b].height;

        if balance > 1 {
            let i_f = b![i_c].child1;
            let i_g = b![i_c].child2;
            //let ref mut f = self.nodes[i_f as usize];
            //let ref mut g = self.nodes[i_g as usize];

            assert!(0 <= i_f && i_f < nodes_len);
            assert!(0 <= i_g && i_g < nodes_len);

            b![i_c].child1 = i_a;
            b![i_c].parent = b![i_a].parent;
            b![i_a].parent = i_c;

            if b![i_c].parent != null_node() {
                if self.nodes[b![i_c].parent as usize].child1 == i_a {
                    let index = b![i_c].parent as usize;
                    self.nodes[index].child1 = i_c;
                }
                else {
                    assert!(self.nodes[b![i_c].parent as usize].child2 == i_a);
                    let index = b![i_c].parent as usize;
                    self.nodes[index].child2 = i_c;
                }
            }
            else {
                self.root = i_c;
            }

            if b![i_f].height > b![i_g].height {
                b![i_c].child2 = i_f;
                b![i_a].child2 = i_g;
                b![i_g].parent = i_a;

                let (temp_aabb1, temp_aabb2) = (b![i_b].aabb, b![i_g].aabb);
                b![i_a].aabb.combine(temp_aabb1, temp_aabb2);
                let (temp_aabb1, temp_aabb2) = (b![i_a].aabb, b![i_f].aabb);
                b![i_c].aabb.combine(temp_aabb1, temp_aabb2);

                b![i_a].height = 1 + max(b![i_b].height, b![i_g].height);
                b![i_c].height = 1 + max(b![i_a].height, b![i_f].height);
            }
            else {
                b![i_c].child2 = i_g;
                b![i_a].child2 = i_f;
                b![i_f].parent = i_a;
                let (temp_aabb1, temp_aabb2) = (b![i_b].aabb, b![i_f].aabb);
                b![i_a].aabb.combine(temp_aabb1, temp_aabb2);
                let (temp_aabb1, temp_aabb2) = (b![i_a].aabb, b![i_g].aabb);
                b![i_c].aabb.combine(temp_aabb1, temp_aabb2);

                b![i_a].height = 1 + max(b![i_b].height, b![i_f].height);
                b![i_c].height = 1 + max(b![i_a].height, b![i_g].height);
            }

            return i_c
        }

        if balance < -1 {
            let i_d = b![i_b].child1;
            let i_e = b![i_b].child2;

            //let ref mut d = self.nodes[i_d as usize];
            //let ref mut e = self.nodes[i_e as usize];

            assert!(0 <= i_d && i_d < self.nodes.len() as i32);
            assert!(0 <= i_e && i_e < self.nodes.len() as i32);

            b![i_b].child1 = i_a;
            b![i_b].parent = b![i_a].parent;
            b![i_a].parent = i_b;

            if b![i_b].parent != null_node() {
                if self.nodes[b![i_b].parent as usize].child1 == i_a {
                    let index = b![i_b].parent as usize;
                    self.nodes[index].child1 = i_b;
                }
                else {
                    assert!(self.nodes[b![i_b].parent as usize].child2 == i_a);
                    let index = b![i_b].parent as usize;
                    self.nodes[index].child2 = i_b;
                }
            }
            else {
                self.root = i_b;
            }

            if b![i_d].height > b![i_e].height {
                b![i_b].child2 = i_d;
                b![i_a].child1 = i_e;
                b![i_e].parent = i_a;

                let (temp_aabb1, temp_aabb2) = (b![i_c].aabb, b![i_e].aabb);
                b![i_a].aabb.combine(temp_aabb1, temp_aabb2);

                let (temp_aabb1, temp_aabb2) = (b![i_a].aabb, b![i_d].aabb);
                b![i_b].aabb.combine(temp_aabb1, temp_aabb2);

                b![i_a].height = 1 + max(b![i_c].height, b![i_e].height);
                b![i_b].height = 1 + max(b![i_a].height, b![i_d].height);
            }
            else {
                b![i_b].child2 = i_e;
                b![i_a].child1 = i_d;
                b![i_d].parent = i_a;

                let (temp_aabb1, temp_aabb2) = (b![i_c].aabb, b![i_d].aabb);
                b![i_a].aabb.combine(temp_aabb1, temp_aabb2);
                let (temp_aabb1, temp_aabb2) = (b![i_a].aabb, b![i_e].aabb);
                b![i_b].aabb.combine(temp_aabb1, temp_aabb2);

                b![i_a].height = 1 + max(b![i_c].height, b![i_c].height);
                b![i_b].height = 1 + max(b![i_a].height, b![i_e].height);
            }
            return i_b
        }
        i_a
    }

    fn extend_aabb(aabb: &mut Aabb, aabb_extension: f32) {
        let r = Vector3::new(aabb_extension,aabb_extension,aabb_extension);

        let vec1 = aabb.min - r;
        let vec2 = aabb.max + r;

        aabb.min = Vector3::new(vec1.x, vec1.y, vec1.z);
        aabb.max = Vector3::new(vec2.x, vec2.y, vec2.z);
    }

    fn user_data(&self, proxy_id: i32) -> Option<UserData> {
        assert!(0 <= proxy_id && proxy_id < self.nodes.len() as i32); //Invalid proxy_id

        self.nodes[proxy_id as usize].user_data
    }

    fn fat_aabb(&self, proxy_id: i32) -> &Aabb {
        assert!(0 <= proxy_id && proxy_id < self.nodes.len() as i32); //Invalid proxy_id

        &self.nodes.get(proxy_id as usize).unwrap().aabb
    }

    fn query<T: FnMut(i32) -> bool>(&self, aabb: Aabb, callback: &mut T) {
        let mut stack: Vec<i32> = Vec::new();
        stack.push(self.root);

        while let Some(node_id) = stack.pop() {
            if node_id == null_node() {
                continue;
            }

            let ref node = self.nodes.get(node_id as usize)
                .expect("Invalid node ID");

            if node.aabb.overlaps(aabb) {
                if node.is_leaf() {
                    if !callback(node_id) { break }
                }
                else {
                    stack.push(node.child1);
                    stack.push(node.child2);
                }
            }
        }
    }

    //TODO fix Fn parameter, is it really needed?
    fn query_frustum<T: FnMut(i32) -> bool>(&self, matrix: luck_math::Matrix4<f32>, _: &mut T) -> Vec<Option<UserData>> {
        let mut ret = vec![];

        let mut planes = [
            /*LEFT*/
            luck_math::Vector4::new(matrix.c0.w + matrix.c0.x,
                                    matrix.c1.w + matrix.c1.x,
                                    matrix.c2.w + matrix.c2.x,
                                    matrix.c3.w + matrix.c3.x),
            /*RIGHT*/
            luck_math::Vector4::new(matrix.c0.w - matrix.c0.x,
                                    matrix.c1.w - matrix.c1.x,
                                    matrix.c2.w - matrix.c2.x,
                                    matrix.c3.w - matrix.c3.x),
            /*BOTTOM*/
            luck_math::Vector4::new(matrix.c0.w + matrix.c0.y,
                                    matrix.c1.w + matrix.c1.y,
                                    matrix.c2.w + matrix.c2.y,
                                    matrix.c3.w + matrix.c3.y),
            /*TOP*/
            luck_math::Vector4::new(matrix.c0.w - matrix.c0.y,
                                    matrix.c1.w - matrix.c1.y,
                                    matrix.c2.w - matrix.c2.y,
                                    matrix.c3.w - matrix.c3.y),
            /*FAR*/
            luck_math::Vector4::new(matrix.c0.w - matrix.c0.z,
                                    matrix.c1.w - matrix.c1.z,
                                    matrix.c2.w - matrix.c2.z,
                                    matrix.c3.w - matrix.c3.z),
            /*NEAR*/
            luck_math::Vector4::new(matrix.c0.w + matrix.c0.z,
                                    matrix.c1.w + matrix.c1.z,
                                    matrix.c2.w + matrix.c2.z,
                                    matrix.c3.w + matrix.c3.z)
        ];

        for i in 0..6 {
            let invl = (planes[i].x * planes[i].x +
                        planes[i].y * planes[i].y +
                        planes[i].z * planes[i].z).sqrt();
            planes[i] = planes[i] / invl;
        }

        let mut stack = vec![];
        stack.push(self.root);

        while let Some(node_id) = stack.pop() {
            if node_id == null_node() {
                continue
            }

            let ref node = self.nodes[node_id as usize];
            let pos = node.aabb.center();

            let result : luck_math::FrustumTestResult = luck_math::is_box_in_frustum(pos, node.aabb.diagonal() / 2.0, planes);

            if result == luck_math::FrustumTestResult::INSIDE {
                let mut all = vec![];
                all.push(node_id);

                while let Some(cur) = all.pop() {
                    let ref cur = self.nodes[cur as usize];
                    if cur.is_leaf() {
                        ret.push(cur.user_data);
                    }
                    else {
                        all.push(cur.child1);
                        all.push(cur.child2);
                    }
                }
            }
            else if result == luck_math::FrustumTestResult::INTERSECT {
                if node.is_leaf() {
                    ret.push(node.user_data);
                }
                else {
                    stack.push(node.child1);
                    stack.push(node.child2);
                }
            }
        }

        ret
    }
}

pub enum FrustumPlanes {
    LEFT = 0, RIGHT = 1, BOTTOM = 2, TOP = 3, FAR = 4, NEAR = 5
}

#[cfg(test)]
mod tests {

    type UserData = i32;
    use super::{DynamicTree};
    use luck_math::*;

    fn aabb(min: (f32, f32, f32), max: (f32, f32, f32)) -> Aabb {
        Aabb::new(Vector3::new(min.0,min.1,min.2), Vector3::new(max.0,max.1,max.2))
    }

    fn aabb_at_pos(pos: (f32,f32,f32)) -> Aabb {
        let mut ret = aabb((-1.0,-1.0,-1.0), (1.0,1.0,1.0));
        ret.translate(Vector3::new(pos.0,pos.1,pos.2));
        ret
    }

    #[test]
    fn creation_querying_destruction() {
        let mut tree = DynamicTree::<UserData>::new();

        let p1 = tree.create_proxy(aabb_at_pos((0.0,0.0,0.0)), 1);
        let _ = tree.create_proxy(aabb_at_pos((10.0,0.0,0.0)), 1);
        let p3 = tree.create_proxy(aabb_at_pos((20.0,0.0,0.0)), 1);
        let _ = tree.create_proxy(aabb_at_pos((30.0,0.0,0.0)), 1);

        fn query_and_return_amount(tree: &DynamicTree<UserData>, aabb: Aabb) -> i32 {
            let mut count = 0;
            tree.query(aabb, &mut |_|{
                count += 1;
                true
            });
            count
        }

        assert_eq!(query_and_return_amount(&tree, aabb((-1.0, -1.0, -1.0),(1.0, 1.0, 1.0))),  1);
        assert_eq!(query_and_return_amount(&tree, aabb((-1.0, -1.0, -1.0),(11.0, 1.0, 1.0))), 2);
        assert_eq!(query_and_return_amount(&tree, aabb((-1.0, -1.0, -1.0),(21.0, 1.0, 1.0))), 3);
        assert_eq!(query_and_return_amount(&tree, aabb((-1.0, -1.0, -1.0),(31.0, 1.0, 1.0))), 4);
        assert_eq!(query_and_return_amount(&tree, aabb((2.0, -1.0, -1.0),(31.0, 1.0, 1.0))),  3);

        tree.destroy_proxy(p1);
        assert_eq!(query_and_return_amount(&tree, aabb((-1.0, -1.0, -1.0),(31.0, 1.0, 1.0))), 3);

        let aabb1 = aabb_at_pos((50.0,0.0,0.0));
        tree.move_proxy(p3, aabb1, Vector3::new(0.0,0.0,0.0));

        assert_eq!(query_and_return_amount(&tree, aabb((2.0, -1.0, -1.0),(31.0, 1.0, 1.0))),  2);
        assert_eq!(query_and_return_amount(&tree, aabb((2.0, -1.0, -1.0),(51.0, 1.0, 1.0))),  3);
    }
}
