use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    model::{BufferLocation, Model, ModelState},
    picking::{Hitbox, HitboxNode, IntoHitbox},
    Rotate, Scale, Translate,
};

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

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate> Model<T, StaticAllocHandle<T>>
    for TreeModel<T, C, StaticAllocHandle<T>>
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

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate> Model<T, DynamicAllocHandle<T>>
    for TreeModel<T, C, DynamicAllocHandle<T>>
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

impl<T, C: Hitbox, H: AllocHandle<T>> IntoHitbox<C> for TreeModel<T, C, H> {
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
