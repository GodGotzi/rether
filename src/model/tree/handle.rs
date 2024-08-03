use crate::{
    alloc::BufferAllocationID,
    model::{BufferLocation, Handle},
    picking::{Hitbox, HitboxNode, IntoHitbox},
    Rotate, Scale, Transform, Translate,
};

#[derive(Debug, Clone)]
pub enum TreeHandle<C> {
    Root {
        id: BufferAllocationID,
        transform: Transform,
        sub_handles: Vec<TreeHandle<C>>,
        ctx: C,
    },
    Node {
        location: BufferLocation,
        sub_handles: Vec<TreeHandle<C>>,
        ctx: C,
    },
}

impl<C: Translate + Scale + Rotate> Handle for TreeHandle<C> {
    fn id(&self) -> &BufferAllocationID {
        match self {
            Self::Root { id, .. } => id,
            Self::Node { .. } => panic!("Cannot get id from node"),
        }
    }

    fn transform(&self) -> &Transform {
        match self {
            Self::Root { transform, .. } => transform,
            Self::Node { .. } => panic!("Cannot get transform from node"),
        }
    }
}

impl<C: Translate> Translate for TreeHandle<C> {
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Root {
                transform,
                sub_handles,
                ..
            } => {
                transform.translate(translation);
                for handle in sub_handles.iter_mut() {
                    handle.translate(translation);
                }
            }
            Self::Node { ctx, .. } => {
                ctx.translate(translation);
            }
        }
    }
}

impl<C: Rotate> Rotate for TreeHandle<C> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Root {
                transform,
                sub_handles,
                ..
            } => {
                transform.rotate(rotation);
                for handle in sub_handles.iter_mut() {
                    handle.rotate(rotation);
                }
            }
            Self::Node { ctx, .. } => {
                ctx.rotate(rotation);
            }
        }
    }
}

impl<C: Scale> Scale for TreeHandle<C> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Root {
                transform,
                sub_handles,
                ..
            } => {
                transform.scale(scale);
                for handle in sub_handles.iter_mut() {
                    handle.scale(scale);
                }
            }
            Self::Node { ctx, .. } => {
                ctx.scale(scale);
            }
        }
    }
}

impl<C: Hitbox> IntoHitbox<C> for TreeHandle<C> {
    fn into_hitbox(self) -> crate::picking::HitboxNode<C> {
        match self {
            Self::Root {
                ctx, sub_handles, ..
            } => HitboxNode::parent_box(
                ctx,
                sub_handles
                    .into_iter()
                    .map(|handle| handle.into_hitbox())
                    .collect(),
            ),
            Self::Node {
                ctx, sub_handles, ..
            } => {
                if sub_handles.is_empty() {
                    HitboxNode::Box { ctx }
                } else {
                    HitboxNode::parent_box(
                        ctx,
                        sub_handles
                            .into_iter()
                            .map(|handle| handle.into_hitbox())
                            .collect(),
                    )
                }
            }
        }
    }
}
