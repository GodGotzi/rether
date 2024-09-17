use std::sync::Arc;

use geometry::IndexedGeometry;
use parking_lot::RwLock;

use crate::{alloc::AllocHandle, Rotate, Scale, SimpleGeometry, Transform, Translate};

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

impl<T, H> From<SimpleGeometry<T>> for ModelState<T, H> {
    fn from(geometry: SimpleGeometry<T>) -> Self {
        Self::Dormant(geometry)
    }
}

impl<T, H> From<IndexedGeometry<T>> for ModelState<T, H> {
    fn from(geometry: IndexedGeometry<T>) -> Self {
        Self::DormantIndexed(geometry)
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

pub trait TransformModel: TranslateModel + RotateModel + ScaleModel {}

pub trait Model<T: Translate + Rotate + Scale, H: AllocHandle<T>>:
    TranslateModel + RotateModel + ScaleModel
{
    fn wake(&self, handle: Arc<H>);

    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }
    fn transform(&self) -> Transform;
    fn state(&self) -> &RwLock<ModelState<T, H>>;
}

pub trait IndexedModel<T: Translate + Rotate + Scale, H: AllocHandle<T>>:
    TranslateModel + RotateModel + ScaleModel
{
    fn wake(&self, handle: Arc<H>, index_handle: Arc<H>);
    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn transform(&self) -> Transform;
    fn state(&self) -> &RwLock<ModelState<T, H>>;
}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}
