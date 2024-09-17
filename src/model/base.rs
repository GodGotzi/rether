use core::panic;

use glam::Vec3;
use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    vertex::{Vertex, VertexRotator, VertexScaler},
    SimpleGeometry,
};

use super::{
    geometry::IndexedGeometry,
    transform::{Rotate, Scale, Translate},
    Model, ModelState, RotateModel, ScaleModel, TranslateModel,
};

#[derive(Debug)]
pub struct BaseModel<T, H: AllocHandle<T>> {
    state: RwLock<ModelState<T, H>>,
    transform: RwLock<crate::Transform>,
}

impl<T, H> BaseModel<T, H>
where
    H: AllocHandle<T>,
{
    pub fn simple(geometry: SimpleGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::Dormant(geometry)),
            transform: RwLock::new(crate::Transform::default()),
        }
    }

    pub fn indexed(geometry: IndexedGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::DormantIndexed(geometry)),
            transform: RwLock::new(crate::Transform::default()),
        }
    }
}

impl Model<Vertex, StaticAllocHandle<Vertex>> for BaseModel<Vertex, StaticAllocHandle<Vertex>>
// C: Translate + Scale + Rotate,
{
    fn wake(&self, handle: std::sync::Arc<StaticAllocHandle<Vertex>>) {
        *self.state.write() = ModelState::Awake(handle);
    }

    fn transform(&self) -> crate::Transform {
        self.transform.read().clone()
    }

    fn state(&self) -> &RwLock<ModelState<Vertex, StaticAllocHandle<Vertex>>> {
        &self.state
    }

    fn destroy(&self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl Model<Vertex, DynamicAllocHandle<Vertex>> for BaseModel<Vertex, DynamicAllocHandle<Vertex>> {
    fn wake(&self, handle: std::sync::Arc<DynamicAllocHandle<Vertex>>) {
        *self.state.write() = ModelState::Awake(handle);
    }

    fn transform(&self) -> crate::Transform {
        self.transform.read().clone()
    }

    fn state(&self) -> &RwLock<ModelState<Vertex, DynamicAllocHandle<Vertex>>> {
        &self.state
    }

    fn destroy(&self) {
        match &*self.state.read() {
            ModelState::Awake(ref handle) => {
                handle.destroy();
            }
            _ => panic!("Cannot destroy a dead handle"),
        };

        *self.state.write() = ModelState::Destroyed;
    }

    fn is_destroyed(&self) -> bool {
        self.state.read().is_destroyed()
    }
}

// Translate, Rotate and Scale are implemented for BaseModel
impl<T, H> TranslateModel for BaseModel<T, H>
where
    T: Translate,
    H: AllocHandle<T>,
{
    fn translate(&self, translation: glam::Vec3) {
        match &mut *self.state.write() {
            ModelState::Awake(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [T]| data.translate(translation));

                let action = ModifyAction::new(0, handle.size(), mod_action);

                self.transform.write().translate(translation);
                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                self.transform.write().translate(translation);
                geometry.translate(translation);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                self.transform.write().translate(translation);
                geometry.translate(translation);
            }
            _ => panic!("Cannot translate a dead handle"),
        }
    }
}

impl<H> RotateModel for BaseModel<Vertex, H>
where
    H: AllocHandle<Vertex>,
{
    fn rotate(&self, rotation: glam::Quat, center: Option<Vec3>) {
        match &mut *self.state.write() {
            ModelState::Awake(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [Vertex]| {
                    VertexRotator::new(data, center.unwrap_or(Vec3::ZERO)).rotate(rotation)
                });

                let action = ModifyAction::new(0, handle.size(), mod_action);

                self.transform.write().rotate(rotation);
                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                self.transform.write().rotate(rotation);
                geometry.rotate(rotation);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                self.transform.write().rotate(rotation);
                geometry.rotate(rotation);
            }
            _ => panic!("Cannot rotate a dead handle"),
        }
    }
}

impl<H> ScaleModel for BaseModel<Vertex, H>
where
    H: AllocHandle<Vertex>,
{
    fn scale(&self, scale: glam::Vec3, center: Option<Vec3>) {
        match &mut *self.state.write() {
            ModelState::Awake(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [Vertex]| {
                    VertexScaler::new(data, center.unwrap_or(Vec3::ZERO)).scale(scale);
                });

                let action = ModifyAction::new(0, handle.size(), mod_action);

                self.transform.write().scale(scale);
                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                self.transform.write().scale(scale);
                geometry.scale(scale);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                self.transform.write().scale(scale);
                geometry.scale(scale);
            }
            _ => panic!("Cannot scale a dead handle"),
        }
    }
}
