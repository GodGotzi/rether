use std::sync::Arc;

use geometry::IndexedGeometry;
use transform::{Rotate, Scale, Translate};

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

pub trait Model<T: Translate + Rotate + Scale, H: AllocHandle<T>>:
    Translate + Rotate + Scale
{
    fn wake(&mut self, handle: Arc<H>);

    fn destroy(&mut self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn state(&self) -> &ModelState<T, H>;
}

pub trait IndexedModel<T: Translate + Rotate + Scale, H: AllocHandle<T>>:
    Translate + Rotate + Scale
{
    fn wake(&self, handle: Arc<H>, index_handle: Arc<H>);
    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn state(&self) -> &ModelState<T, H>;
}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}
