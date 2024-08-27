use core::panic;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    picking::{interact::Interactive, Hitbox, HitboxNode},
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

impl<T, C, H> BaseModel<T, C, H>
where
    H: AllocHandle<T>,
{
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

impl<T, C, H> HitboxNode<BaseModel<T, C, H>> for BaseModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate + Hitbox,
    H: AllocHandle<T>,
{
    fn check_hit(&self, ray: &crate::picking::Ray) -> Option<f32> {
        self.ctx.check_hit(ray)
    }

    fn inner_nodes(&self) -> &[BaseModel<T, C, H>] {
        &[]
    }
}

impl<T, C, H> Interactive for BaseModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate + Interactive,
    H: AllocHandle<T>,
{
    fn mouse_clicked(&mut self, button: winit::event::MouseButton) {
        self.ctx.mouse_clicked(button);
    }

    fn mouse_motion(&mut self, button: winit::event::MouseButton, delta: glam::Vec2) {
        self.ctx.mouse_motion(button, delta);
    }

    fn mouse_scroll(&mut self, delta: f32) {
        self.ctx.mouse_scroll(delta);
    }
}

impl<T, C> Model<T, StaticAllocHandle<T>> for BaseModel<T, C, StaticAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
{
    fn wake(&mut self, handle: std::sync::Arc<StaticAllocHandle<T>>) {
        self.state = ModelState::Awake(handle);
    }

    fn state(&self) -> &ModelState<T, StaticAllocHandle<T>> {
        &self.state
    }

    fn destroy(&mut self) {
        panic!("Static handle cannot be destroyed");
    }
}

impl<T, C> Model<T, DynamicAllocHandle<T>> for BaseModel<T, C, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
{
    fn wake(&mut self, handle: std::sync::Arc<DynamicAllocHandle<T>>) {
        self.state = ModelState::Awake(handle);
    }

    fn state(&self) -> &ModelState<T, DynamicAllocHandle<T>> {
        &self.state
    }

    fn destroy(&mut self) {
        match self.state {
            ModelState::Awake(ref handle) => {
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
impl<T, C, H> Translate for BaseModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn translate(&mut self, translation: glam::Vec3) {
        match self.state {
            ModelState::Awake(ref mut handle) => {
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

impl<T, C, H> Rotate for BaseModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn rotate(&mut self, rotation: glam::Quat) {
        self.ctx.rotate(rotation);
    }
}

impl<T, C, H> Scale for BaseModel<T, C, H>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
    H: AllocHandle<T>,
{
    fn scale(&mut self, scale: glam::Vec3) {
        self.ctx.scale(scale);
    }
}
