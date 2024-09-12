use core::panic;

use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
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
}

impl<T, H> BaseModel<T, H>
where
    H: AllocHandle<T>,
{
    pub fn simple(geometry: SimpleGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::Dormant(geometry)),
        }
    }

    pub fn indexed(geometry: IndexedGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::DormantIndexed(geometry)),
        }
    }
}

impl<T> Model<T, StaticAllocHandle<T>> for BaseModel<T, StaticAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    // C: Translate + Scale + Rotate,
{
    fn wake(&self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        *self.state.write() = ModelState::Awake(handle);
    }

    fn state(&self) -> &RwLock<ModelState<T, StaticAllocHandle<T>>> {
        &self.state
    }

    fn destroy(&self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<T> Model<T, DynamicAllocHandle<T>> for BaseModel<T, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    // C: Translate + Scale + Rotate,
{
    fn wake(&self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        *self.state.write() = ModelState::Awake(handle);
    }

    fn state(&self) -> &RwLock<ModelState<T, DynamicAllocHandle<T>>> {
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

                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                geometry.translate(translation);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                geometry.translate(translation);
            }
            _ => panic!("Cannot translate a dead handle"),
        }
    }
}

impl<T, H> RotateModel for BaseModel<T, H>
where
    T: Rotate,
    H: AllocHandle<T>,
{
    fn rotate(&self, rotation: glam::Quat) {
        match &mut *self.state.write() {
            ModelState::Awake(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [T]| data.rotate(rotation));

                let action = ModifyAction::new(0, handle.size(), mod_action);

                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                geometry.rotate(rotation);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                geometry.rotate(rotation);
            }
            _ => panic!("Cannot rotate a dead handle"),
        }
    }
}

impl<T, H> ScaleModel for BaseModel<T, H>
where
    T: Scale,
    H: AllocHandle<T>,
{
    fn scale(&self, scale: glam::Vec3) {
        match &mut *self.state.write() {
            ModelState::Awake(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [T]| data.scale(scale));

                let action = ModifyAction::new(0, handle.size(), mod_action);

                handle.send_action(action).expect("Failed to send action");
            }
            ModelState::Dormant(ref mut geometry) => {
                geometry.scale(scale);
            }
            ModelState::DormantIndexed(ref mut geometry) => {
                geometry.scale(scale);
            }
            _ => panic!("Cannot scale a dead handle"),
        }
    }
}
