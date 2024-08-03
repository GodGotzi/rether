mod hitbox;
mod queue;
mod ray;

pub use hitbox::{Hitbox, HitboxNode};

pub use ray::Ray;

pub struct PickingContext<C> {
    hitbox: HitboxNode<C>,
}

impl<C: Hitbox> Default for PickingContext<C> {
    fn default() -> Self {
        PickingContext {
            hitbox: HitboxNode::root(),
        }
    }
}

impl<C: Hitbox> PickingContext<C> {
    pub fn add_hitbox(&mut self, node: HitboxNode<C>) {
        self.hitbox.add_node(node);
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<&C> {
        self.hitbox.check_hit(ray)
    }
}
