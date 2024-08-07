use std::sync::Arc;

use geometry::Geometry;
use transform::{Rotate, Scale, Translate};

use crate::{
    alloc::{AllocHandle, BufferAllocationID, ModifyAction},
    Transform,
};

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
pub enum ModelState<G: Geometry, H> {
    Dormant(G),
    Alive(Arc<H>),
    Destroyed,
}

impl<G: Geometry, H> ModelState<G, H> {
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Alive(_))
    }

    pub fn is_destroyed(&self) -> bool {
        matches!(self, Self::Destroyed)
    }
}

pub trait Model<T: Translate + Rotate + Scale, G: Geometry, H: AllocHandle<T>>:
    Translate + Rotate + Scale
{
    fn make_alive(&mut self, handle: Arc<H>);
    fn destroy(&mut self) {}
    fn is_destroyed(&self) -> bool {
        false
    }

    fn state(&self) -> &ModelState<G, H>;
}

pub trait IndexedModel<T: Translate + Rotate + Scale, H: AllocHandle<T>>:
    Translate + Rotate + Scale
{
    fn make_alive(&self, handle: Arc<H>, index_handle: Arc<H>);
    fn destroy(&self) {}
    fn is_destroyed(&self) -> bool {
        false
    }
    fn raw_handle(&self) -> Arc<H>;
    fn raw_index_handle(&self) -> Arc<H>;

    fn translate(&self, translation: glam::Vec3) {
        let handle = self.raw_handle();

        let mod_action = Box::new(move |data: &mut [T]| {
            for item in data.iter_mut() {
                item.translate(translation);
            }
        });

        let action = ModifyAction::new(0, handle.size(), mod_action);

        handle.send_action(action).expect("Failed to send action");
    }
}

pub trait Handle: Translate + Rotate + Scale {
    fn id(&self) -> &BufferAllocationID;
    fn transform(&self) -> &Transform;
}

pub trait ModelContext: Expandable {}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}
