use core::panic;

use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    model::{BufferLocation, Model, ModelState},
    Rotate, Scale, Transform, Translate,
};

use super::{RotateModel, ScaleModel, TranslateModel};
// rethink tree cause usage is pretty complicated
#[derive(Debug)]
pub enum TreeModel<S, T, H: AllocHandle<T>> {
    Root {
        state: RwLock<ModelState<T, H>>,
        transform: RwLock<Transform>,
        sub_handles: Vec<S>,
    },
    Node {
        location: BufferLocation,
        sub_handles: Vec<S>,
    },
}

impl<S, T, H> TreeModel<S, T, H>
where
    H: AllocHandle<T>,
{
    pub fn create_root<M: Into<ModelState<T, H>>>(geometry: M) -> Self {
        Self::Root {
            state: RwLock::new(geometry.into()),
            transform: RwLock::new(Transform::default()),
            sub_handles: Vec::new(),
        }
    }

    pub fn create_root_with_models<M: Into<ModelState<T, H>>>(
        geometry: M,
        sub_handles: Vec<S>,
    ) -> Self {
        Self::Root {
            state: RwLock::new(geometry.into()),
            transform: RwLock::new(Transform::default()),
            sub_handles,
        }
    }

    pub fn create_node(location: BufferLocation) -> Self {
        Self::Node {
            location,
            sub_handles: Vec::new(),
        }
    }

    pub fn create_node_with_models(location: BufferLocation, sub_handles: Vec<S>) -> Self {
        Self::Node {
            location,
            sub_handles,
        }
    }
}

/*
impl<T, H> TreeModel<T, H>
where
    T: Clone,
    H: AllocHandle<T>,
{
    pub fn add_child(&mut self, other: Self) {
        match self {
            TreeModel::Root {
                state: self_state,
                sub_handles: self_sub_handles,
                ctx: self_ctx,
                ..
            } => match other {
                TreeModel::Root {
                    mut state,
                    sub_handles,
                    mut ctx,
                    ..
                } => {
                    let (offset, size) = match self_state.get_mut() {
                        ModelState::Dormant(geometry) => match state.get_mut() {
                            ModelState::Dormant(other_geometry) => {
                                let offset = geometry.data_len();

                                geometry.expand(other_geometry);

                                (offset, other_geometry.data_len())
                            }
                            ModelState::DormantIndexed(_) => {
                                panic!("Cannot expand a dormant geometry with an indexed geometry");
                            }
                            _ => panic!("Cannot expand an alive or dead handle"),
                        },
                        ModelState::DormantIndexed(geometry) => match state.get_mut() {
                            ModelState::Dormant(_) => {
                                panic!("Cannot expand a dormant geometry with an indexed geometry");
                            }
                            ModelState::DormantIndexed(other_geometry) => {
                                let offset = geometry.data_len();

                                geometry.expand(other_geometry);

                                (offset, other_geometry.data_len())
                            }
                            _ => panic!("Cannot expand an alive or dead handle"),
                        },
                        _ => panic!("Cannot expand an alive or dead handle"),
                    };

                    self_ctx.get_mut().expand_hitbox(ctx.get_mut());

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
                    mut ctx,
                } => {
                    self_ctx.get_mut().expand_hitbox(ctx.get_mut());

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
                    mut ctx,
                } => {
                    self_ctx.get_mut().expand_hitbox(ctx.get_mut());

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
*/

impl<S, T> Model<T, StaticAllocHandle<T>> for TreeModel<S, T, StaticAllocHandle<T>> {
    fn wake(&self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state.write() = ModelState::Awake(handle);
            }
            Self::Node { .. } => {
                panic!("Cannot make alive a node");
            }
        }
    }

    fn state(&self) -> &RwLock<ModelState<T, StaticAllocHandle<T>>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } => panic!("Cannot get state from node"),
        }
    }

    fn destroy(&self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<S, T> Model<T, DynamicAllocHandle<T>> for TreeModel<S, T, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
{
    fn wake(&self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state.write() = ModelState::Awake(handle);
            }
            Self::Node { .. } => {
                panic!("Cannot make alive a node");
            }
        }
    }

    fn state(&self) -> &RwLock<ModelState<T, DynamicAllocHandle<T>>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } => panic!("Cannot get state from node"),
        }
    }

    fn destroy(&self) {
        match self {
            Self::Root { state, .. } => {
                match &*state.read() {
                    ModelState::Awake(handle) => {
                        handle.destroy();
                    }
                    _ => panic!("Cannot destroy a dead handle"),
                }

                *state.write() = ModelState::Destroyed;
            }
            Self::Node { .. } => {
                panic!("Cannot destroy a node");
            }
        }
    }

    fn is_destroyed(&self) -> bool {
        match self {
            Self::Root { state, .. } => state.read().is_destroyed(),
            Self::Node { .. } => {
                panic!("Cannot check if a node is destroyed");
            }
        }
    }
}

impl<S: TranslateModel, T: Translate, H: AllocHandle<T>> TranslateModel for TreeModel<S, T, H> {
    fn translate(&self, translation: glam::Vec3) {
        match self {
            Self::Root {
                state,
                sub_handles,
                transform,
            } => {
                transform.write().translate(translation);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action =
                            Box::new(move |data: &mut [T]| data.translate(translation));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.translate(translation);

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.translate(translation);

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    _ => panic!("Cannot translate a dead handle"),
                }
            }
            Self::Node { sub_handles, .. } => {
                for handle in sub_handles.iter() {
                    handle.translate(translation);
                }
            }
        }
    }
}

impl<S: RotateModel, T: Rotate, H: AllocHandle<T>> RotateModel for TreeModel<S, T, H> {
    fn rotate(&self, rotation: glam::Quat) {
        match self {
            Self::Root {
                state,
                sub_handles,
                transform,
                ..
            } => {
                transform.write().rotate(rotation);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action = Box::new(move |data: &mut [T]| data.rotate(rotation));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    _ => panic!("Cannot rotate a dead handle"),
                }
            }
            Self::Node { sub_handles, .. } => {
                for handle in sub_handles.iter() {
                    handle.rotate(rotation);
                }
            }
        }
    }
}

impl<S: ScaleModel, T: Scale, H: AllocHandle<T>> ScaleModel for TreeModel<S, T, H> {
    fn scale(&self, scale: glam::Vec3) {
        match self {
            Self::Root {
                state,
                sub_handles,
                transform,
                ..
            } => {
                transform.write().scale(scale);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action = Box::new(move |data: &mut [T]| data.scale(scale));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    _ => panic!("Cannot scale a dead handle"),
                }
            }
            Self::Node { sub_handles, .. } => {
                for handle in sub_handles.iter() {
                    handle.scale(scale);
                }
            }
        }
    }
}
