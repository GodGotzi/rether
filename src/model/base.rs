use core::panic;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    SimpleGeometry,
};

use super::{
    geometry::IndexedGeometry,
    transform::{Rotate, Scale, Translate},
    Model, ModelState,
};

pub struct BaseModel<T, C, H: AllocHandle<T>> {
    state: ModelState<T, H>,
    ctx: C,
}

impl<T, C, H: AllocHandle<T>> BaseModel<T, C, H> {
    pub fn simple(ctx: C, geometry: SimpleGeometry<T>) -> Self {
        Self {
            state: ModelState::Dormant(geometry),
            ctx,
        }
    }

    pub fn indexed(ctx: C, geometry: IndexedGeometry<T>) -> Self {
        Self {
            state: ModelState::DormantIndexed(geometry),
            ctx,
        }
    }
}

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate> Model<T, StaticAllocHandle<T>>
    for BaseModel<T, C, StaticAllocHandle<T>>
{
    fn make_alive(&mut self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        self.state = ModelState::Alive(handle);
    }

    fn state(&self) -> &ModelState<T, StaticAllocHandle<T>> {
        &self.state
    }

    fn destroy(&mut self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate> Model<T, DynamicAllocHandle<T>>
    for BaseModel<T, C, DynamicAllocHandle<T>>
{
    fn make_alive(&mut self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        self.state = ModelState::Alive(handle);
    }

    fn state(&self) -> &ModelState<T, DynamicAllocHandle<T>> {
        &self.state
    }

    fn destroy(&mut self) {
        match self.state {
            ModelState::Alive(ref handle) => {
                handle.destroy();
            }
            _ => panic!("Cannot destroy a dead handle"),
        };

        self.state = ModelState::Destroyed;
    }

    fn is_destroyed(&self) -> bool {
        self.state.is_destroyed()
    }
}

// Translate, Rotate and Scale are implemented for BaseModel
impl<C, T, H> Translate for BaseModel<T, C, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn translate(&mut self, translation: glam::Vec3) {
        match self.state {
            ModelState::Alive(ref mut handle) => {
                let mod_action = Box::new(move |data: &mut [T]| data.translate(translation));

                let action = ModifyAction::new(0, handle.size(), mod_action);

                handle.send_action(action).expect("Failed to send action");

                self.ctx.translate(translation);
            }
            ModelState::Dormant(ref mut geometry) => {
                geometry.translate(translation);

                self.ctx.translate(translation);
            }
            _ => panic!("Cannot translate a dead handle"),
        }
    }
}

impl<C, T, H> Rotate for BaseModel<T, C, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn rotate(&mut self, rotation: glam::Quat) {
        self.ctx.rotate(rotation);
    }
}

impl<C, T, H> Scale for BaseModel<T, C, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn scale(&mut self, scale: glam::Vec3) {
        self.ctx.scale(scale);
    }
}
