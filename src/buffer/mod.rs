pub mod alloc;
mod raw;

use std::sync::Arc;

use alloc::{AllocHandle, DynamicAllocHandle};

use raw::*;
use wgpu::{Device, Queue};

pub struct BufferData<'a, T> {
    data: &'a [T],
}

impl<'a, T> BufferData<'a, T> {
    pub fn create(data: &'a [T]) -> Self {
        Self { data }
    }
}

pub struct IndexedBufferData<'a, T> {
    data: &'a [T],
    indices: &'a [u32],
}

impl<'a, T> IndexedBufferData<'a, T> {
    pub fn create(vertices: &'a [T], indices: &'a [u32]) -> Self {
        Self {
            data: vertices,
            indices,
        }
    }
}

#[derive(Debug)]
pub struct Buffer<T, L> {
    inner: RawBuffer,
    allocater: Box<L>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable, L: alloc::BufferAlloc<T>> Buffer<T, L> {
    pub fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut wgpu::RenderPass<'b>) {
        render_pass.set_vertex_buffer(0, self.inner.inner.slice(..));
        render_pass.draw(self.inner.render_range.clone(), 0..1);
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable, L: alloc::BufferAlloc<T> + Default> Buffer<T, L> {
    pub fn new(label: &str, device: &wgpu::Device) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        // default allocater
        let allocater = L::default();

        let inner =
            RawBuffer::new::<T>(allocater.size(), label, wgpu::BufferUsages::VERTEX, device);

        Self {
            inner,
            allocater: Box::new(allocater),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn write(&self, id: &str, buffer_data: BufferData<'_, T>, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Some(allocation) = self.allocater.get(id) {
            // assert!(buffer_data.data.len() <= allocation.size());

            self.inner
                .write(queue, allocation.offset(), buffer_data.data);
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable, L: alloc::BufferDynamicAlloc<T>> Buffer<T, L> {
    pub fn allocate<const S: usize>(
        &mut self,
        id: &str,
        device: &Device,
        queue: &Queue,
    ) -> Arc<DynamicAllocHandle<T>>
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let handle = self.allocater.allocate(id, S);

        self.inner.allocate::<T>(S, device, queue);

        handle
    }

    pub fn allocate_init(
        &mut self,
        id: &str,
        buffer_data: BufferData<'_, T>,
        device: &Device,
        queue: &Queue,
    ) -> Arc<DynamicAllocHandle<T>>
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let handle = self.allocater.allocate(id, buffer_data.data.len());

        self.inner.append(buffer_data.data, device, queue);

        handle
    }

    pub fn free(&mut self, id: &str, device: &Device, queue: &Queue) {
        if let Some(allocation) = self.allocater.free(id) {
            self.inner
                .free::<T>(allocation.offset, allocation.size, device, queue);
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        self.allocater
            .update(|mod_action| self.inner.modify(mod_action, device, queue));

        for id in self.allocater.get_destroyed_handles() {
            self.free(&id, device, queue);
        }
    }
}

#[derive(Debug)]
pub struct IndexedBuffer<T, L, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    L: alloc::BufferAlloc<T>,
    I: alloc::BufferAlloc<T>,
{
    inner: RawBuffer,
    index: RawBuffer,
    allocater: Box<L>,
    allocator_index: Box<I>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, L, I> IndexedBuffer<T, L, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    L: alloc::BufferAlloc<T>,
    I: alloc::BufferAlloc<T>,
{
    pub fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut wgpu::RenderPass<'b>) {
        render_pass.set_vertex_buffer(0, self.inner.inner.slice(..));
        render_pass.set_index_buffer(self.index.inner.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index.size as u32, 0, 0..1);
    }
}

impl<T, L, I> IndexedBuffer<T, L, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    L: alloc::BufferAlloc<T> + Default,
    I: alloc::BufferAlloc<T> + Default,
{
    pub fn new(label: &str, device: &wgpu::Device) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let allocater = L::default();
        let allocator_index = I::default();

        let inner =
            RawBuffer::new::<T>(allocater.size(), label, wgpu::BufferUsages::VERTEX, device);
        let index = RawBuffer::new::<u32>(
            allocator_index.size(),
            &format!("Index {}", label),
            wgpu::BufferUsages::VERTEX,
            device,
        );

        Self {
            inner,
            index,
            allocater: Box::new(allocater),
            allocator_index: Box::new(allocator_index),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, L, I> IndexedBuffer<T, L, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    L: alloc::BufferAlloc<T> + Default,
    I: alloc::BufferAlloc<T> + Default,
{
    pub fn write<const DS: usize, const IS: usize>(
        &self,
        id: &str,
        buffer_data: IndexedBufferData<'_, T>,
        queue: &wgpu::Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Some(allocation) = self.allocater.get(id) {
            self.inner
                .write(queue, allocation.offset(), buffer_data.data);
        }

        if let Some(allocation) = self.allocator_index.get(id) {
            self.index
                .write(queue, allocation.offset(), buffer_data.indices);
        }
    }
}

impl<T, L, I> IndexedBuffer<T, L, I>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
    L: alloc::BufferDynamicAlloc<T>,
    I: alloc::BufferDynamicAlloc<T>,
{
    pub fn allocate<const DS: usize, const IS: usize>(
        &mut self,
        id: &str,
        device: &Device,
        queue: &Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        self.allocater.allocate(id, DS);
        self.allocator_index.allocate(id, IS);

        self.inner.allocate::<T>(DS, device, queue);
        self.index.allocate::<u32>(IS, device, queue);
    }

    pub fn allocate_init(
        &mut self,
        id: &str,
        buffer_data: IndexedBufferData<'_, T>,
        device: &Device,
        queue: &Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        self.allocater.allocate(id, buffer_data.data.len());
        self.allocator_index.allocate(id, buffer_data.indices.len());

        self.inner.append(buffer_data.data, device, queue);
        self.index.append(buffer_data.data, device, queue);
    }

    pub fn free(&mut self, id: &str, device: &Device, queue: &Queue) {
        if let Some(allocation) = self.allocater.free(id) {
            self.inner
                .free::<T>(allocation.offset, allocation.size, device, queue);
        }

        if let Some(allocation) = self.allocator_index.free(id) {
            self.index
                .free::<u32>(allocation.offset, allocation.size, device, queue);
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        self.allocater
            .update(|mod_action| self.inner.modify(mod_action, device, queue));

        let mut pending_destroyed_handles = self.allocater.get_destroyed_handles();

        self.allocator_index.update(|_| {});

        pending_destroyed_handles.extend(self.allocator_index.get_destroyed_handles());

        for id in pending_destroyed_handles {
            self.free(&id, device, queue);
        }
    }
}
