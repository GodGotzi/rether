use core::panic;

use crate::alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle};

use super::{
    geometry::Geometry,
    transform::{Rotate, Scale, Translate},
    Model, ModelState,
};

pub struct BaseModel<T, C, G: Geometry, H: AllocHandle<T>> {
    state: ModelState<G, H>,
    ctx: C,
    _phantom: std::marker::PhantomData<T>,
}

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate, G: Geometry>
    Model<T, G, StaticAllocHandle<T>> for BaseModel<T, C, G, StaticAllocHandle<T>>
{
    fn make_alive(&mut self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        self.state = ModelState::Alive(handle);
    }

    fn state(&self) -> &ModelState<G, StaticAllocHandle<T>> {
        &self.state
    }

    fn destroy(&mut self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<C: Translate + Scale + Rotate, T: Translate + Scale + Rotate, G: Geometry>
    Model<T, G, DynamicAllocHandle<T>> for BaseModel<T, C, G, DynamicAllocHandle<T>>
{
    fn make_alive(&mut self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        self.state = ModelState::Alive(handle);
    }

    fn state(&self) -> &ModelState<G, DynamicAllocHandle<T>> {
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
impl<C, T, G, H> Translate for BaseModel<T, C, G, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    G: Geometry,
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

impl<C, T, G, H> Rotate for BaseModel<T, C, G, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    G: Geometry,
    H: AllocHandle<T>,
{
    fn rotate(&mut self, rotation: glam::Quat) {
        self.ctx.rotate(rotation);
    }
}

impl<C, T, G, H> Scale for BaseModel<T, C, G, H>
where
    C: Translate + Scale + Rotate,
    T: Translate + Scale + Rotate,
    G: Geometry,
    H: AllocHandle<T>,
{
    fn scale(&mut self, scale: glam::Vec3) {
        self.ctx.scale(scale);
    }
}
