use transform::{Rotate, Scale, Transform, Translate};

use crate::{
    buffer::alloc::BufferAllocationID,
    picking::{
        hitbox::{Hitbox, HitboxNode, InteractContext},
        interactive::Interactive,
    },
};

pub mod transform;

#[derive(Debug, Clone)]
pub struct BufferLocation {
    pub offset: usize,
    pub size: usize,
}

impl<T: Translate> Translate for [T] {
    fn translate(&mut self, translation: glam::Vec3) {
        for item in self.iter_mut() {
            item.translate(translation);
        }
    }
}

impl<T: Rotate> Rotate for [T] {
    fn rotate(&mut self, rotation: glam::Quat) {
        for item in self.iter_mut() {
            item.rotate(rotation);
        }
    }
}

impl<T: Scale> Scale for [T] {
    fn scale(&mut self, scale: glam::Vec3) {
        for item in self.iter_mut() {
            item.scale(scale);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Geometry<T> {
    Simple { vertices: Vec<T> },
    Indexed { indices: Vec<u32>, vertices: Vec<T> },
}

impl<T: Translate> Translate for Geometry<T> {
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Simple { vertices } => vertices.translate(translation),
            Self::Indexed { vertices, .. } => vertices.translate(translation),
        }
    }
}

impl<T: Rotate> Rotate for Geometry<T> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Simple { vertices } => vertices.rotate(rotation),
            Self::Indexed { vertices, .. } => vertices.rotate(rotation),
        }
    }
}

impl<T: Scale> Scale for Geometry<T> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Simple { vertices } => vertices.scale(scale),
            Self::Indexed { vertices, .. } => vertices.scale(scale),
        }
    }
}

pub trait IntoHandle<T> {
    fn req_handle(self, allocation_id: BufferAllocationID) -> T;
}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}

// maybe fix update bug
#[derive(Debug, Clone)]
pub enum TreeModel<T, C> {
    Root {
        geometry: Geometry<T>,
        sub_models: Vec<TreeModel<T, C>>,
        ctx: C,
    },
    Node {
        location: BufferLocation,
        sub_models: Vec<TreeModel<T, C>>,
        ctx: C,
    },
}

impl<T, C: Expandable> TreeModel<T, C> {
    pub fn expand(&mut self, model: TreeModel<T, C>) {
        match self {
            Self::Root {
                sub_models, ctx, ..
            } => {
                ctx.expand(model.ctx());
                sub_models.push(model);
            }
            Self::Node {
                sub_models, ctx, ..
            } => {
                ctx.expand(model.ctx());
                sub_models.push(model);
            }
        }
    }

    pub fn ctx(&self) -> &C {
        match self {
            Self::Root { ctx, .. } => ctx,
            Self::Node { ctx, .. } => ctx,
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone, C> IntoHandle<TreeHandle<C>>
    for TreeModel<T, C>
{
    fn req_handle(self, allocation_id: BufferAllocationID) -> TreeHandle<C> {
        match self {
            Self::Root {
                sub_models, ctx, ..
            } => TreeHandle::Root {
                id: allocation_id.clone(),
                transform: Transform::default(),
                sub_handles: sub_models
                    .into_iter()
                    .map(|model| model.req_handle(allocation_id.clone()))
                    .collect(),
                ctx,
            },
            Self::Node {
                location,
                sub_models,
                ctx,
            } => TreeHandle::Node {
                location: location.clone(),
                sub_handles: sub_models
                    .into_iter()
                    .map(|model| model.req_handle(allocation_id.clone()))
                    .collect(),
                ctx,
            },
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Translate, C: Translate> Translate
    for TreeModel<T, C>
{
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                ctx.translate(translation);
                geometry.translate(translation);
                for model in sub_models.iter_mut() {
                    model.translate(translation);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.translate(translation);
                for model in sub_models.iter_mut() {
                    model.translate(translation);
                }
            }
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Rotate, C: Rotate> Rotate for TreeModel<T, C> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                geometry.rotate(rotation);
                ctx.rotate(rotation);
                for model in sub_models.iter_mut() {
                    model.rotate(rotation);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.rotate(rotation);
                for model in sub_models.iter_mut() {
                    model.rotate(rotation);
                }
            }
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Scale, C: Scale> Scale for TreeModel<T, C> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Root {
                geometry,
                sub_models,
                ctx,
                ..
            } => {
                geometry.scale(scale);
                ctx.scale(scale);
                for model in sub_models.iter_mut() {
                    model.scale(scale);
                }
            }
            Self::Node {
                ctx, sub_models, ..
            } => {
                ctx.scale(scale);
                for model in sub_models.iter_mut() {
                    model.scale(scale);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TreeHandle<C> {
    Root {
        id: BufferAllocationID,
        transform: transform::Transform,
        sub_handles: Vec<TreeHandle<C>>,
        ctx: C,
    },
    Node {
        location: BufferLocation,
        sub_handles: Vec<TreeHandle<C>>,
        ctx: C,
    },
}

impl Into<HitboxNode> for TreeHandle<InteractContext> {
    fn into(self) -> HitboxNode {
        match self {
            Self::Root {
                ctx, sub_handles, ..
            } => HitboxNode::parent_box(
                ctx,
                sub_handles
                    .into_iter()
                    .map(|handle| handle.into())
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
                            .map(|handle| handle.into())
                            .collect(),
                    )
                }
            }
        }
    }
}

impl<C: Interactive + Hitbox> Interactive for TreeHandle<C> {
    fn mouse_clicked(&mut self, button: winit::event::MouseButton) {
        match self {
            Self::Root { ctx, .. } => ctx.mouse_clicked(button),
            Self::Node { ctx, .. } => ctx.mouse_clicked(button),
        }
    }

    fn mouse_motion(&mut self, button: winit::event::MouseButton, delta: glam::Vec2) {
        match self {
            Self::Root { ctx, .. } => ctx.mouse_motion(button, delta),
            Self::Node { ctx, .. } => ctx.mouse_motion(button, delta),
        }
    }

    fn mouse_scroll(&mut self, delta: f32) {
        match self {
            Self::Root { ctx, .. } => ctx.mouse_scroll(delta),
            Self::Node { ctx, .. } => ctx.mouse_scroll(delta),
        }
    }
}

impl<C: Hitbox> Hitbox for TreeHandle<C> {
    fn check_hit(&self, ray: &crate::picking::ray::Ray) -> Option<f32> {
        match self {
            Self::Root { ctx, .. } => ctx.check_hit(ray),
            Self::Node { ctx, .. } => ctx.check_hit(ray),
        }
    }

    fn expand(&mut self, _box: &dyn Hitbox) {
        match self {
            Self::Root { ctx, .. } => ctx.expand(_box),
            Self::Node { ctx, .. } => ctx.expand(_box),
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        match self {
            Self::Root { ctx, .. } => ctx.set_enabled(enabled),
            Self::Node { ctx, .. } => ctx.set_enabled(enabled),
        }
    }

    fn enabled(&self) -> bool {
        match self {
            Self::Root { ctx, .. } => ctx.enabled(),
            Self::Node { ctx, .. } => ctx.enabled(),
        }
    }

    fn min(&self) -> glam::Vec3 {
        match self {
            Self::Root { ctx, .. } => ctx.min(),
            Self::Node { ctx, .. } => ctx.min(),
        }
    }

    fn max(&self) -> glam::Vec3 {
        match self {
            Self::Root { ctx, .. } => ctx.max(),
            Self::Node { ctx, .. } => ctx.max(),
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
