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
    Leaf {
        location: BufferLocation,
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

    pub fn sub_handles(&self) -> Option<&Vec<S>> {
        match self {
            Self::Root { sub_handles, .. } => Some(sub_handles),
            Self::Node { sub_handles, .. } => Some(sub_handles),
            Self::Leaf { .. } => None,
        }
    }
}

impl<S: TranslateModel + RotateModel + ScaleModel, T: Translate + Rotate + Scale>
    Model<T, StaticAllocHandle<T>> for TreeModel<S, T, StaticAllocHandle<T>>
{
    fn wake(&self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state.write() = ModelState::Awake(handle);
            }
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot wake a node or leaf");
            }
        }
    }

    fn transform(&self) -> Transform {
        match self {
            Self::Root { transform, .. } => transform.read().clone(),
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot get transform from node or leaf")
            }
        }
    }

    fn state(&self) -> &RwLock<ModelState<T, StaticAllocHandle<T>>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } | Self::Leaf { .. } => panic!("Cannot get state from node or leaf"),
        }
    }

    fn destroy(&self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<S: TranslateModel + RotateModel + ScaleModel, T: Translate + Rotate + Scale>
    Model<T, DynamicAllocHandle<T>> for TreeModel<S, T, DynamicAllocHandle<T>>
{
    fn wake(&self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        match self {
            Self::Root { state, .. } => {
                *state.write() = ModelState::Awake(handle);
            }
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot wake a node or leaf");
            }
        }
    }

    fn transform(&self) -> Transform {
        match self {
            Self::Root { transform, .. } => transform.read().clone(),
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot get transform from node or leaf")
            }
        }
    }

    fn state(&self) -> &RwLock<ModelState<T, DynamicAllocHandle<T>>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } | Self::Leaf { .. } => panic!("Cannot get state from node or leaf"),
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
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot destroy a node or leaf");
            }
        }
    }

    fn is_destroyed(&self) -> bool {
        match self {
            Self::Root { state, .. } => state.read().is_destroyed(),
            Self::Node { .. } | Self::Leaf { .. } => {
                panic!("Cannot check if a node or leaf is destroyed");
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
            _ => {}
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
            _ => {}
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
            _ => {}
        }
    }
}
