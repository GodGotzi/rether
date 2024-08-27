use std::sync::Arc;

use geometry::IndexedGeometry;
use glam::Vec2;
use parking_lot::RwLock;
use winit::event::MouseButton;

use crate::{alloc::AllocHandle, SimpleGeometry};

mod base;
pub mod geometry;
pub mod transform;
mod tree;

pub use base::BaseModel;
pub use tree::TreeModel;

#[derive(Debug, Clone)]
pub struct BufferLocation {
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub enum ModelState<T, H> {
    Dormant(SimpleGeometry<T>),
    DormantIndexed(IndexedGeometry<T>),
    Awake(Arc<H>),
    Destroyed,
}

impl<T, H> ModelState<T, H> {
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Awake(_))
    }

    pub fn is_destroyed(&self) -> bool {
        matches!(self, Self::Destroyed)
    }
}

pub trait TranslateModel {
    fn translate(&self, translation: glam::Vec3);
}

pub trait RotateModel {
    fn rotate(&self, rotation: glam::Quat);
}

pub trait ScaleModel {
    fn scale(&self, scale: glam::Vec3);
}

pub trait InteractiveModel {
    fn mouse_clicked(&self, button: MouseButton);
    fn mouse_scroll(&self, delta: f32);
    fn mouse_motion(&self, button: MouseButton, delta: Vec2);
}

pub trait Model<T, H: AllocHandle<T>> {
    fn wake(&self, handle: Arc<H>);

    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn state(&self) -> &RwLock<ModelState<T, H>>;
}

pub trait IndexedModel<T, H: AllocHandle<T>> {
    fn wake(&self, handle: Arc<H>, index_handle: Arc<H>);
    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn state(&self) -> &RwLock<ModelState<T, H>>;
}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}
