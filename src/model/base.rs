use core::panic;

use parking_lot::RwLock;

use crate::{
    alloc::{AllocHandle, DynamicAllocHandle, ModifyAction, StaticAllocHandle},
    picking::{
        interact::{Interactive, InteractiveModel},
        Hitbox, HitboxNode,
    },
    SimpleGeometry,
};

use super::{
    geometry::IndexedGeometry,
    transform::{Rotate, Scale, Translate},
    Model, ModelState, RotateModel, ScaleModel, TranslateModel,
};

#[derive(Debug)]
pub struct BaseModel<T, C, H: AllocHandle<T>> {
    state: RwLock<ModelState<T, H>>,
    ctx: RwLock<C>,
}

impl<T, C, H> BaseModel<T, C, H>
where
    H: AllocHandle<T>,
{
    pub fn simple(ctx: C, geometry: SimpleGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::Dormant(geometry)),
            ctx: RwLock::new(ctx),
        }
    }

    pub fn indexed(ctx: C, geometry: IndexedGeometry<T>) -> Self {
        Self {
            state: RwLock::new(ModelState::DormantIndexed(geometry)),
            ctx: RwLock::new(ctx),
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
        self.ctx.read().check_hit(ray)
    }

    fn inner_nodes(&self) -> &[BaseModel<T, C, H>] {
        &[]
    }

    fn get_max(&self) -> glam::Vec3 {
        self.ctx.read().get_max()
    }

    fn get_min(&self) -> glam::Vec3 {
        self.ctx.read().get_min()
    }
}

impl<T, C, H> InteractiveModel for BaseModel<T, C, H>
where
    C: Interactive<Model = BaseModel<T, C, H>>,
    H: AllocHandle<T>,
{
    fn clicked(&self, event: crate::picking::interact::ClickEvent) {
        self.ctx.write().clicked(event)(self);
    }

    fn drag(&self, event: crate::picking::interact::DragEvent) {
        self.ctx.write().drag(event)(self);
    }

    fn scroll(&self, event: crate::picking::interact::ScrollEvent) {
        self.ctx.write().scroll(event)(self);
    }
}

impl<T, C> Model<T, StaticAllocHandle<T>> for BaseModel<T, C, StaticAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
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

impl<T, C> Model<T, DynamicAllocHandle<T>> for BaseModel<T, C, DynamicAllocHandle<T>>
where
    T: Translate + Scale + Rotate,
    C: Translate + Scale + Rotate,
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
impl<T, C, H> TranslateModel for BaseModel<T, C, H>
where
    T: Translate,
    C: Translate,
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

        self.ctx.write().translate(translation);
    }
}

impl<T, C, H> RotateModel for BaseModel<T, C, H>
where
    T: Rotate,
    C: Rotate,
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

        self.ctx.write().rotate(rotation);
    }
}

impl<T, C, H> ScaleModel for BaseModel<T, C, H>
where
    T: Scale,
    C: Scale,
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

        self.ctx.write().scale(scale);
    }
}
