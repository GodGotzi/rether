use std::sync::Arc;

use glam::Vec3;

use super::{
    queue::{HitBoxQueueEntry, HitboxQueue},
    ray::Ray,
};

pub trait Hitbox: std::fmt::Debug + Send + Sync {
    fn check_hit(&self, ray: &Ray) -> Option<f32>;
    fn expand_hitbox(&mut self, _box: &dyn Hitbox);
    fn set_enabled(&mut self, enabled: bool);
    fn enabled(&self) -> bool;
    fn get_min(&self) -> Vec3;
    fn get_max(&self) -> Vec3;
}

pub trait HitboxNode<M: HitboxNode<M>> {
    fn check_hit(&self, ray: &Ray) -> Option<f32>;
    fn inner_nodes(&self) -> &[M];
    fn get_min(&self) -> Vec3;
    fn get_max(&self) -> Vec3;
}

// Importing the Ray struct from the ray module in the super namespace
// Function to check if a ray hits a hitbox node, returning an optional usize

// Definition of the HitboxNode enum with Debug trait
#[derive(Debug, Clone)]
pub struct HitboxRoot<M: HitboxNode<M>> {
    inner_hitboxes: Vec<Arc<M>>,
}

// Implementation of methods for HitboxNode
impl<M: HitboxNode<M>> HitboxRoot<M> {
    pub fn root() -> Self {
        Self {
            inner_hitboxes: Vec::new(),
        }
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<&M> {
        let mut queue = HitboxQueue::<M>::new(); // Creating a new HitboxQueue

        for hitbox in self.inner_hitboxes.iter() {
            let distance = hitbox.check_hit(ray);
            if let Some(distance) = distance {
                queue.push(HitBoxQueueEntry { hitbox, distance });
            }
        }

        while let Some(HitBoxQueueEntry { hitbox, .. }) = queue.pop() {
            if hitbox.inner_nodes().is_empty() {
                return Some(hitbox);
            } else {
                for inner_hitbox in hitbox.inner_nodes() {
                    let distance = inner_hitbox.check_hit(ray);
                    if let Some(distance) = distance {
                        queue.push(HitBoxQueueEntry {
                            hitbox: inner_hitbox,
                            distance,
                        });
                    }
                }
            }
        }

        None
    }

    pub fn add_node(&mut self, node: Arc<M>) {
        self.inner_hitboxes.push(node);
    }
}

/*
// Test function for hitbox functionality
#[test]
pub fn test_hitbox() {
    use glam::vec3;
    use glam::Vec3;

    let mut root = HitboxNode::parent_box(BoundingBox::default()); // Creating a default HitBoxRoot

    let box_ = HitboxNode::box_(
        BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0)), // Creating a bounding box with specific dimensions
        Arc::new(0),
    );

    root.add_hitbox(box_); // Adding the box to the root

    let ray = Ray {
        origin: Vec3::new(0.0, 0.0, 0.0),
        direction: Vec3::new(1.0, 1.0, 1.0),
    };

    let hit = root.check_hit(&ray); // Checking if the ray hits the box

    assert_eq!(hit, Some(30)); // Asserting that the hit id is 30
}

// Test function for hitbox parent functionality
#[test]
pub fn test_hitbox_parent() {
    use glam::vec3;
    use glam::Vec3; // Importing Vec3 from glam crate

    let mut root = HitboxNode::parent_box(BoundingBox::default()); // Creating a default HitBoxRoot

    let mut parent =
        HitboxNode::parent_box(BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0))); // Creating a parent box with specific dimensions

    let box_ = HitboxNode::box_(
        BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(0.5, 0.5, 0.5)), // Creating a smaller bounding box
        30,
    );

    parent.add_hitbox(box_); // Adding the smaller box to the parent box

    root.add_hitbox(parent); // Adding the parent box to the root

    let ray = Ray {
        origin: Vec3::new(0.0, 0.0, 0.0),
        direction: Vec3::new(1.0, 1.0, 1.0),
    };

    let hit = root.check_hit(&ray); // Checking if the ray hits any of the boxes

    assert_eq!(hit, Some(30)); // Asserting that the hit id is 30
}
*/
