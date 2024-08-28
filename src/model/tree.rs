use core::panic;

use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    model::{BufferLocation, Model, ModelState},
    picking::{
        interact::{Interactive, InteractiveModel},
        Hitbox, HitboxNode,
    },
    Rotate, Scale, Transform, Translate,
};

use super::{geometry::Geometry, Expandable, RotateModel, ScaleModel, TranslateModel};

#[derive(Debug)]
pub enum TreeModel<T, C, H: AllocHandle<T>> {
    Root {
        state: RwLock<ModelState<T, H>>,
        transform: RwLock<Transform>,
        sub_handles: Vec<TreeModel<T, C, H>>,
        ctx: RwLock<C>,
    },
    Node {
        location: BufferLocation,
        sub_handles: Vec<TreeModel<T, C, H>>,
        ctx: RwLock<C>,
    },
}

impl<T, C, H> TreeModel<T, C, H>
where
    H: AllocHandle<T>,
{
    pub fn create_root<M: Into<ModelState<T, H>>>(ctx: C, geometry: M) -> Self {
        Self::Root {
            state: RwLock::new(geometry.into()),
            transform: RwLock::new(Transform::default()),
            sub_handles: Vec::new(),
            ctx: RwLock::new(ctx),
        }
    }

    pub fn create_root_with_models<M: Into<ModelState<T, H>>>(
        ctx: C,
        geometry: M,
        sub_handles: Vec<Self>,
    ) -> Self {
        Self::Root {
            state: RwLock::new(geometry.into()),
            transform: RwLock::new(Transform::default()),
            sub_handles,
            ctx: RwLock::new(ctx),
        }
    }

    pub fn create_node(ctx: C, location: BufferLocation) -> Self {
        Self::Node {
            location,
            sub_handles: Vec::new(),
            ctx: RwLock::new(ctx),
        }
    }

    pub fn create_node_with_models(
        ctx: C,
        location: BufferLocation,
        sub_handles: Vec<Self>,
    ) -> Self {
        Self::Node {
            location,
            sub_handles,
            ctx: RwLock::new(ctx),
        }
    }
}

impl<T, C, H> TreeModel<T, C, H>
where
    T: Clone,
    C: Hitbox,
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

impl<T, C> Model<T, StaticAllocHandle<T>> for TreeModel<T, C, StaticAllocHandle<T>> {
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

impl<T, C> Model<T, DynamicAllocHandle<T>> for TreeModel<T, C, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
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

impl<T, C, H> HitboxNode<TreeModel<T, C, H>> for TreeModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate + Hitbox,
    H: AllocHandle<T>,
{
    fn check_hit(&self, ray: &crate::picking::Ray) -> Option<f32> {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => ctx.read().check_hit(ray),
        }
    }

    fn inner_nodes(&self) -> &[TreeModel<T, C, H>] {
        match self {
            Self::Root { sub_handles, .. } => sub_handles,
            Self::Node { sub_handles, .. } => sub_handles,
        }
    }

    fn get_max(&self) -> glam::Vec3 {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => ctx.read().get_max(),
        }
    }

    fn get_min(&self) -> glam::Vec3 {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => ctx.read().get_min(),
        }
    }
}

impl<T, C, H> InteractiveModel for TreeModel<T, C, H>
where
    C: Interactive,
    H: AllocHandle<T>,
{
    fn clicked(&self, event: crate::picking::interact::ClickEvent) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.write().clicked(event);
            }
        }
    }

    fn drag(&self, event: crate::picking::interact::DragEvent) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.write().drag(event);
            }
        }
    }

    fn scroll(&self, event: crate::picking::interact::ScrollEvent) {
        match self {
            Self::Root { ctx, .. } | Self::Node { ctx, .. } => {
                ctx.write().scroll(event);
            }
        }
    }
}

impl<T: Translate, C: Translate, H: AllocHandle<T>> TranslateModel for TreeModel<T, C, H> {
    fn translate(&self, translation: glam::Vec3) {
        match self {
            Self::Root {
                state,
                sub_handles,
                ctx,
                transform,
                ..
            } => {
                transform.write().translate(translation);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action =
                            Box::new(move |data: &mut [T]| data.translate(translation));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        ctx.write().translate(translation);

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.translate(translation);

                        ctx.write().translate(translation);

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.translate(translation);

                        ctx.write().translate(translation);

                        for handle in sub_handles.iter() {
                            handle.translate(translation);
                        }
                    }
                    _ => panic!("Cannot translate a dead handle"),
                }
            }
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.write().translate(translation);

                for handle in sub_handles.iter() {
                    handle.translate(translation);
                }
            }
        }
    }
}

impl<T: Rotate, C: Rotate, H: AllocHandle<T>> RotateModel for TreeModel<T, C, H> {
    fn rotate(&self, rotation: glam::Quat) {
        match self {
            Self::Root {
                state,
                sub_handles,
                ctx,
                transform,
                ..
            } => {
                transform.write().rotate(rotation);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action = Box::new(move |data: &mut [T]| data.rotate(rotation));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        ctx.write().rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.rotate(rotation);

                        ctx.write().rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.rotate(rotation);

                        ctx.write().rotate(rotation);

                        for handle in sub_handles.iter() {
                            handle.rotate(rotation);
                        }
                    }
                    _ => panic!("Cannot rotate a dead handle"),
                }
            }
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.write().rotate(rotation);

                for handle in sub_handles.iter() {
                    handle.rotate(rotation);
                }
            }
        }
    }
}

impl<T: Scale, C: Scale, H: AllocHandle<T>> ScaleModel for TreeModel<T, C, H> {
    fn scale(&self, scale: glam::Vec3) {
        match self {
            Self::Root {
                state,
                sub_handles,
                ctx,
                transform,
                ..
            } => {
                transform.write().scale(scale);
                match &mut *state.write() {
                    ModelState::Awake(handle) => {
                        let mod_action = Box::new(move |data: &mut [T]| data.scale(scale));

                        let action = ModifyAction::new(0, handle.size(), mod_action);

                        handle.send_action(action).expect("Failed to send action");

                        ctx.write().scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    ModelState::Dormant(geometry) => {
                        geometry.scale(scale);

                        ctx.write().scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    ModelState::DormantIndexed(geometry) => {
                        geometry.scale(scale);

                        ctx.write().scale(scale);

                        for handle in sub_handles.iter() {
                            handle.scale(scale);
                        }
                    }
                    _ => panic!("Cannot scale a dead handle"),
                }
            }
            Self::Node {
                ctx, sub_handles, ..
            } => {
                ctx.write().scale(scale);

                for handle in sub_handles.iter() {
                    handle.scale(scale);
                }
            }
        }
    }
}
