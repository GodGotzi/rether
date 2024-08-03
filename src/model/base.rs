use crate::{
    alloc::BufferAllocationID,
    picking::{Hitbox, IntoHitbox},
    Geometry,
};

use super::{
    transform::{Rotate, Scale, Transform, Translate},
    Handle, Model,
};

pub struct BaseModel<T, C> {
    geometry: Geometry<T>,
    ctx: C,
}

impl<T, C> Model<T, BaseHandle<C>> for BaseModel<T, C>
where
    C: Translate + Rotate + Scale,
{
    fn geometry(&self) -> &Geometry<T> {
        &self.geometry
    }

    fn into_handle(self, allocation_id: BufferAllocationID) -> BaseHandle<C> {
        BaseHandle {
            id: allocation_id,
            transform: Transform::default(),
            ctx: self.ctx,
        }
    }
}

#[derive(Debug)]
pub struct BaseHandle<C> {
    id: BufferAllocationID,
    transform: Transform,
    ctx: C,
}

impl<C: Translate + Scale + Rotate> Handle for BaseHandle<C> {
    fn id(&self) -> &BufferAllocationID {
        &self.id
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }
}

impl<C: Translate> Translate for BaseHandle<C> {
    fn translate(&mut self, translation: glam::Vec3) {
        self.transform.translate(translation);
        self.ctx.translate(translation);
    }
}

impl<C: Rotate> Rotate for BaseHandle<C> {
    fn rotate(&mut self, rotation: glam::Quat) {
        self.transform.rotate(rotation);
        self.ctx.rotate(rotation);
    }
}

impl<C: Scale> Scale for BaseHandle<C> {
    fn scale(&mut self, scale: glam::Vec3) {
        self.transform.scale(scale);
        self.ctx.scale(scale);
    }
}

impl<C: Hitbox> IntoHitbox<C> for BaseHandle<C> {
    fn into_hitbox(self) -> crate::picking::HitboxNode<C> {
        crate::picking::HitboxNode::Box { ctx: self.ctx }
    }
}
