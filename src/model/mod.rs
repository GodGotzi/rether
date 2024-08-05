use std::sync::atomic::AtomicBool;

use parking_lot::Mutex;
use transform::{Rotate, Scale, Translate};

use crate::{
    alloc::{BufferAllocation, BufferAllocationID},
    Geometry, Transform,
};

mod base;
pub mod geometry;
pub mod transform;
mod tree;

pub use base::{BaseHandle, BaseModel};
pub use tree::{TreeHandle, TreeModel};

pub struct AllocHandle {
    destroyed: AtomicBool,
    allocation: BufferAllocation,
}

#[derive(Debug, Clone)]
pub struct BufferLocation {
    pub offset: usize,
    pub size: usize,
}

pub trait Model<T, H: Handle> {
    fn geometry(&self) -> &Geometry<T>;
    fn into_handle(self, allocation_id: BufferAllocationID) -> H;
}

pub trait Handle: Translate + Rotate + Scale {
    fn id(&self) -> &BufferAllocationID;
    fn transform(&self) -> &Transform;
}

pub trait ModelContext: Expandable {}

pub trait Expandable {
    fn expand(&mut self, other: &Self);
}
