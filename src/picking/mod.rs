mod hitbox;
pub mod interact;
mod queue;
mod ray;

pub use hitbox::{Hitbox, HitboxNode};
pub use ray::Ray;

pub trait IntoHitbox<C> {
    fn into_hitbox(self) -> HitboxNode<C>;
}

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
    pub fn add_handle<H: IntoHitbox<C>>(&mut self, handle: H) {
        let node = handle.into_hitbox();

        self.hitbox.add_node(node);
    }

    pub fn check_hit(&self, ray: &Ray) -> Option<&C> {
        self.hitbox.check_hit(ray)
    }
}
