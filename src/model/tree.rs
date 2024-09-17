use core::panic;

use glam::Vec3;
use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    model::{BufferLocation, Model, ModelState},
    vertex::{Vertex, VertexRotator, VertexScaler},
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

impl<S: TranslateModel + RotateModel + ScaleModel> Model<Vertex, StaticAllocHandle<Vertex>>
    for TreeModel<S, Vertex, StaticAllocHandle<Vertex>>
{
    fn wake(&self, handle: std::sync::Arc<StaticAllocHandle<Vertex>>) {
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

    fn state(&self) -> &RwLock<ModelState<Vertex, StaticAllocHandle<Vertex>>> {
        match self {
            Self::Root { state, .. } => state,
            Self::Node { .. } | Self::Leaf { .. } => panic!("Cannot get state from node or leaf"),
        }
    }

    fn destroy(&self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<S: TranslateModel + RotateModel + ScaleModel> Model<Vertex, DynamicAllocHandle<Vertex>>
    for TreeModel<S, Vertex, DynamicAllocHandle<Vertex>>
{
    fn wake(&self, handle: std::sync::Arc<DynamicAllocHandle<Vertex>>) {
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

    fn state(&self) -> &RwLock<ModelState<Vertex, DynamicAllocHandle<Vertex>>> {
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

impl<S: RotateModel, H: AllocHandle<Vertex>> RotateModel for TreeModel<S, Vertex, H> {
    fn rotate(&self, rotation: glam::Quat, center: Option<glam::Vec3>) {
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
                        let mod_action = Box::new(move |data: &mut [Vertex]| {
                            //data.rotate(rotation);
                            VertexRotator::new(data, center.unwrap_or(Vec3::ZERO)).rotate(rotation);
                        });

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation, center);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation, center);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation, center);
                        }
                    }
                    _ => panic!("Cannot rotate a dead handle"),
                }
            }
            Self::Node { sub_handles, .. } => {
                for handle in sub_handles.iter() {
                    handle.rotate(rotation, center);
                }
            }
            _ => {}
        }
    }
}

impl<S: ScaleModel, H: AllocHandle<Vertex>> ScaleModel for TreeModel<S, Vertex, H> {
    fn scale(&self, scale: glam::Vec3, center: Option<glam::Vec3>) {
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
                        let mod_action = Box::new(move |data: &mut [Vertex]| {
                            VertexScaler::new(data, center.unwrap_or(Vec3::ZERO)).scale(scale);
                        });

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        for handle in sub_handles.iter() {
                            handle.scale(scale, center);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale, center);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale, center);
                        }
                    }
                    _ => panic!("Cannot scale a dead handle"),
                }
            }
            Self::Node { sub_handles, .. } => {
                for handle in sub_handles.iter() {
                    handle.scale(scale, center);
                }
            }
            _ => {}
        }
    }
}
