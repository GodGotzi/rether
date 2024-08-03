use crate::buffer::alloc::BufferAllocationID;

pub mod geometry;
pub mod transform;
pub mod tree;

#[derive(Debug, Clone)]
pub struct BufferLocation {
    pub offset: usize,
    pub size: usize,
}

pub trait IntoHandle<T> {
    fn req_handle(self, allocation_id: BufferAllocationID) -> T;
}

pub trait ModelContext {
    fn expand(&mut self, other: &Self);
}
