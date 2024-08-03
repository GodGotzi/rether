use std::collections::BinaryHeap;

use super::{hitbox::HitboxNode, Hitbox};

pub type HitboxQueue<'a, C> = BinaryHeap<HitBoxQueueEntry<'a, C>>;

#[derive(Debug)]
pub struct HitBoxQueueEntry<'a, C> {
    pub hitbox: &'a HitboxNode<C>,
    pub distance: f32,
}

impl<C: Hitbox> PartialEq for HitBoxQueueEntry<'_, C> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<C: Hitbox> Eq for HitBoxQueueEntry<'_, C> {}

impl<C: Hitbox> PartialOrd for HitBoxQueueEntry<'_, C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Hitbox> Ord for HitBoxQueueEntry<'_, C> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance
            .partial_cmp(&other.distance)
            .unwrap()
            .reverse()
    }
}
