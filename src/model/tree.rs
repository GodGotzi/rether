use core::panic;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    model::{BufferLocation, Model, ModelState},
    picking::{interact::Interactive, Hitbox, HitboxNode},
    Rotate, Scale, Translate,
};

use super::{geometry::Geometry, Expandable};

#[derive(Debug, Clone)]
pub enum TreeModel<T, C, H: AllocHandle<T>> {
    Root {
        state: ModelState<T, H>,
        sub_handles: Vec<TreeModel<T, C, H>>,
        ctx: C,
    },
    Node {
        location: BufferLocation,
        sub_handles: Vec<TreeModel<T, C, H>>,
        ctx: C,
    },
}

impl<T, C, H> TreeModel<T, C, H>
where
    T: Translate + Scale + Rotate + Clone,
    C: Translate + Scale + Rotate + Hitbox,
    H: AllocHandle<T>,
{
    pub fn add_child(&mut self, other: Self) {
        match self {
            TreeModel::Root {
                state: self_state,
                sub_handles: self_sub_handles,
                ctx: self_ctx,
            } => match other {
                TreeModel::Root {
                    state,
                    sub_handles,
                    ctx,
                } => {
                    let (offset, size) = match self_state {
                        ModelState::Dormant(geometry) => match state {
                            ModelState::Dormant(other_geometry) => {
                                let offset = geometry.data_len();

                                geometry.expand(&other_geometry);

                                (offset, other_geometry.data_len())
                            }
                            ModelState::DormantIndexed(_) => {
                                panic!("Cannot expand a dormant geometry with an indexed geometry");
                            }
                            _ => panic!("Cannot expand an alive or dead handle"),
                        },
                        ModelState::DormantIndexed(geometry) => match state {
                            ModelState::Dormant(_) => {
                                panic!("Cannot expand a dormant geometry with an indexed geometry");
                            }
                            ModelState::DormantIndexed(other_geometry) => {
                                let offset = geometry.data_len();

                                geometry.expand(&other_geometry);

                                (offset, other_geometry.data_len())
                            }
                            _ => panic!("Cannot expand an alive or dead handle"),
                        },
                        _ => panic!("Cannot expand an alive or dead handle"),
                    };

                    self_ctx.expand(&ctx);

                    let node = TreeModel::Node {
                        location: BufferLocation { offset, size },
                        sub_handles,
                        ctx,
                    };

                    self_sub_handles.push(node);
                }
                TreeModel::Node {
                    location,
                    sub_handles,
                    ctx,
                } => {
                    self_ctx.expand(&ctx);

                    let node = TreeModel::Node {
                        location,
                        sub_handles,
                        ctx,
                    };

                    self_sub_handles.push(node);
                }
            },
            TreeModel::Node {
                location,
                sub_handles: self_sub_handles,
                ctx: self_ctx,
            } => match other {
                TreeModel::Node {
                    location: mut other_location,
                    sub_handles,
                    ctx,
                } => {
                    self_ctx.expand(&ctx);

                    other_location.offset += location.size;
                    location.size += other_location.size;

                    let node = TreeModel::Node {
                        location: other_location,
                        sub_handles,
                        ctx,
                    };

                    self_sub_handles.push(node);
                }
                _ => panic!("Cannot expand a node with a root"),
            },
        }
    }
}

impl<T, C> Model<T, StaticAllocHandle<T>> for TreeModel<T, C, StaticAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
{
    fn make_alive(&mut self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state = ModelState::Alive(handle);
            }
            Self::Node { .. } => {
                panic!("Cannot make alive a node");
            }
        }
    }

    fn state(&self) -> &ModelState<T, StaticAllocHandle<T>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } => panic!("Cannot get state from node"),
        }
    }

    fn destroy(&mut self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<T, C> Model<T, DynamicAllocHandle<T>> for TreeModel<T, C, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
{
    fn make_alive(&mut self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state = ModelState::Alive(handle);
            }
            Self::Node { .. } => {
                panic!("Cannot make alive a node");
            }
        }
    }

    fn state(&self) -> &crate::model::ModelState<T, DynamicAllocHandle<T>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } => panic!("Cannot get state from node"),
        }
    }

    fn destroy(&mut self) {
        match self {
            Self::Root { state, .. } => {
                match state {
                    ModelState::Alive(handle) => {
                        handle.destroy();
                    }
                    _ => panic!("Cannot destroy a dead handle"),
                }

                *state = ModelState::Destroyed;
            }
            Self::Node { .. } => {
                panic!("Cannot destroy a node");
            }
        }
    }

    fn is_destroyed(&self) -> bool {
        match self {
            Self::Root { state, .. } => state.is_destroyed(),
            Self::Node { .. } => {
                panic!("Cannot check if a node is destroyed");
            }
        }
    }
}

impl<T, C, H> HitboxNode<TreeModel<T, C, H>> for TreeModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate + Hitbox,
    H: AllocHandle<T>,
{
    fn check_hit(&self, ray: &crate::picking::Ray) -> Option<f32> {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => ctx.check_hit(ray),
        }
    }

    fn inner_nodes(&self) -> &[TreeModel<T, C, H>] {
        match self {
            Self::Root { sub_handles, .. } => sub_handles,
            Self::Node { sub_handles, .. } => sub_handles,
        }
    }
}

impl<T, C, H> Interactive for TreeModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate + Interactive,
    H: AllocHandle<T>,
{
    fn mouse_clicked(&mut self, button: winit::event::MouseButton) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.mouse_clicked(button);
            }
        }
    }

    fn mouse_motion(&mut self, button: winit::event::MouseButton, delta: glam::Vec2) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.mouse_motion(button, delta);
            }
        }
    }

    fn mouse_scroll(&mut self, delta: f32) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.mouse_scroll(delta);
            }
        }
    }
}

impl<T: Translate, C: Translate, H: AllocHandle<T>> Translate for TreeModel<T, C, H> {
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Root {
                state,
                sub_handles,
                ctx,
                ..
            } => match state {
                ModelState::Alive(handle) => {
                    let mod_action = Box::new(move |data: &mut [T]| data.translate(translation));

                    let action = ModifyAction::new(0, handle.size(), mod_action);

                    handle.send_action(action).expect("Failed to send action");

                    ctx.translate(translation);

                    for handle in sub_handles.iter_mut() {
                        handle.translate(translation);
                    }
                }
                ModelState::Dormant(geometry) => {
                    geometry.translate(translation);

                    ctx.translate(translation);

                    for handle in sub_handles.iter_mut() {
                        handle.translate(translation);
                    }
                }
                ModelState::DormantIndexed(geometry) => {
                    geometry.translate(translation);

                    ctx.translate(translation);

                    for handle in sub_handles.iter_mut() {
                        handle.translate(translation);
                    }
                }
                _ => panic!("Cannot translate a dead handle"),
            },
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.translate(translation);

                for handle in sub_handles.iter_mut() {
                    handle.translate(translation);
                }
            }
        }
    }
}

impl<T: Rotate, C: Rotate, H: AllocHandle<T>> Rotate for TreeModel<T, C, H> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Root {
                // transform,
                sub_handles,
                ctx,
                ..
            } => {
                // transform.rotate(rotation);
                ctx.rotate(rotation);
                for handle in sub_handles.iter_mut() {
                    handle.rotate(rotation);
                }
            }
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.rotate(rotation);

                for handle in sub_handles.iter_mut() {
                    handle.rotate(rotation);
                }
            }
        }
    }
}

impl<T: Scale, C: Scale, H: AllocHandle<T>> Scale for TreeModel<T, C, H> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Root {
                // transform,
                sub_handles,
                ctx,
                ..
            } => {
                // transform.scale(scale);
                ctx.scale(scale);
                for handle in sub_handles.iter_mut() {
                    handle.scale(scale);
                }
            }
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.scale(scale);

                for handle in sub_handles.iter_mut() {
                    handle.scale(scale);
                }
            }
        }
    }
}
