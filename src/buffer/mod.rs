use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferAddress, BufferDescriptor, Device, Queue,
};

use crate::model::geometry::Geometry;

pub mod alloc;

#[derive(Debug)]
struct RawBuffer {
    inner: wgpu::Buffer,
    render_range: std::ops::Range<u32>,

    usage: wgpu::BufferUsages,

    size: BufferAddress,
    label: String,
}

impl RawBuffer {
    fn new<T>(size: usize, label: &str, usage: wgpu::BufferUsages, device: &wgpu::Device) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let inner = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: (size * std::mem::size_of::<T>()) as BufferAddress,
            usage: usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            inner,
            render_range: 0..size as u32,

            usage,

            size: size as BufferAddress,
            label: label.to_string(),
        }
    }

    fn new_init<T>(
        data: &[T],
        label: &str,
        usage: wgpu::BufferUsages,
        device: &wgpu::Device,
    ) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let inner = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        Self {
            inner,
            render_range: 0..data.len() as u32,

            usage,

            size: data.len() as BufferAddress,
            label: label.to_string(),
        }
    }

    fn allocate<T>(&mut self, size: usize, device: &wgpu::Device, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let old_bytes = self.size * std::mem::size_of::<T>() as BufferAddress;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&self.label),
            size: old_bytes + (size * std::mem::size_of::<T>()) as BufferAddress,
            usage: self.usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });
        encoder.copy_buffer_to_buffer(&self.inner, 0, &buffer, 0, old_bytes);

        queue.submit(std::iter::once(encoder.finish()));

        self.inner.destroy();

        self.inner = buffer;

        self.size += size as BufferAddress;
        self.render_range = 0..self.size as u32;
    }

    fn append<T>(&mut self, data: &[T], device: &wgpu::Device, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let old_bytes = self.size * std::mem::size_of::<T>() as BufferAddress;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&self.label),
            size: old_bytes + std::mem::size_of_val(data) as BufferAddress,
            usage: self.usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });
        encoder.copy_buffer_to_buffer(&self.inner, 0, &buffer, 0, old_bytes);

        queue.submit(std::iter::once(encoder.finish()));

        queue.write_buffer(&buffer, old_bytes, bytemuck::cast_slice(data));

        self.inner.destroy();

        self.inner = buffer;

        self.size += data.len() as BufferAddress;
        self.render_range = 0..self.size as u32;
    }

    fn free<T>(&mut self, offset: usize, size: usize, device: &wgpu::Device, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let old_bytes = self.size * std::mem::size_of::<T>() as BufferAddress;

        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(&self.label),
            size: old_bytes - (size * std::mem::size_of::<T>()) as BufferAddress,
            usage: self.usage | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let byte_offset = offset * std::mem::size_of::<T>();
        let byte_size_to_free = size * std::mem::size_of::<T>();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(&self.inner, 0, &buffer, 0, byte_offset as BufferAddress);

        encoder.copy_buffer_to_buffer(
            &self.inner,
            (byte_offset + byte_size_to_free) as BufferAddress,
            &buffer,
            byte_offset as BufferAddress,
            old_bytes - (byte_offset + byte_size_to_free) as BufferAddress,
        );

        queue.submit(std::iter::once(encoder.finish()));

        self.inner.destroy();

        self.inner = buffer;

        self.size -= size as BufferAddress;
        self.render_range = 0..self.size as u32;
    }

    fn write<T>(&self, queue: &wgpu::Queue, offset: usize, data: &[T])
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let offset_bytes = offset * std::mem::size_of::<T>();

        queue.write_buffer(&self.inner, offset_bytes as u64, bytemuck::cast_slice(data));
    }
}

#[derive(Debug)]
pub struct Buffer<T, L> {
    raw: RawBuffer,
    allocater: Box<L>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable, L: alloc::BufferAlloc<T>> Buffer<T, L> {
    pub fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut wgpu::RenderPass<'b>) {
        render_pass.set_vertex_buffer(0, self.raw.inner.slice(..));
        render_pass.draw(self.raw.render_range.clone(), 0..1);
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
            raw: inner,
            allocater: Box::new(allocater),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn write(&self, id: &str, geometry: &Geometry<T>, queue: &wgpu::Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Geometry::Simple { vertices } = geometry {
            if let Some(allocation) = self.allocater.get(id) {
                self.raw.write(queue, allocation.offset, vertices);
            }
        }
    }
}

impl<T: bytemuck::Pod + bytemuck::Zeroable, L: alloc::BufferDynamicAlloc<T>> Buffer<T, L> {
    pub fn allocate<const S: usize>(&mut self, id: &str, device: &Device, queue: &Queue)
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        self.allocater.allocate(id, S);

        self.raw.allocate::<T>(S, device, queue);
    }

    pub fn allocate_init(
        &mut self,
        id: &str,
        geometry: &Geometry<T>,
        device: &Device,
        queue: &Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Geometry::Simple { vertices } = geometry {
            self.allocater.allocate(id, vertices.len());

            self.raw.append(vertices, device, queue);
        }
    }

    pub fn free(&mut self, id: &str, device: &Device, queue: &Queue) {
        if let Some(allocation) = self.allocater.free(id) {
            self.raw
                .free::<T>(allocation.offset, allocation.size, device, queue);
        }
    }
}

#[derive(Debug)]
pub struct IndexedBuffer<T, L, I> {
    inner: RawBuffer,
    index: RawBuffer,
    allocater: Box<L>,
    allocator_index: Box<I>,
    _phantom: std::marker::PhantomData<T>,
}

impl<
        T: bytemuck::Pod + bytemuck::Zeroable,
        L: alloc::BufferAlloc<T>,
        I: alloc::BufferAlloc<u32>,
    > IndexedBuffer<T, L, I>
{
    pub fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut wgpu::RenderPass<'b>) {
        render_pass.set_vertex_buffer(0, self.inner.inner.slice(..));
        render_pass.set_index_buffer(self.index.inner.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index.size as u32, 0, 0..1);
    }
}

impl<
        T: bytemuck::Pod + bytemuck::Zeroable,
        L: alloc::BufferAlloc<T> + Default,
        I: alloc::BufferAlloc<u32> + Default,
    > IndexedBuffer<T, L, I>
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

impl<
        T: bytemuck::Pod + bytemuck::Zeroable,
        L: alloc::BufferAlloc<T>,
        I: alloc::BufferAlloc<u32>,
    > IndexedBuffer<T, L, I>
{
    pub fn writ<const DS: usize, const IS: usize>(
        &self,
        id: &str,
        geometry: &Geometry<T>,
        queue: &wgpu::Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Geometry::Indexed { indices, vertices } = geometry {
            if let Some(allocation) = self.allocater.get(id) {
                self.inner.write(queue, allocation.offset, vertices);
            }

            if let Some(allocation) = self.allocator_index.get(id) {
                self.index.write(queue, allocation.offset, indices);
            }
        }
    }
}

impl<
        T: bytemuck::Pod + bytemuck::Zeroable,
        L: alloc::BufferDynamicAlloc<T>,
        I: alloc::BufferDynamicAlloc<u32>,
    > IndexedBuffer<T, L, I>
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
        geometry: &Geometry<T>,
        device: &Device,
        queue: &Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        if let Geometry::Indexed { indices, vertices } = geometry {
            self.allocater.allocate(id, vertices.len());
            self.allocator_index.allocate(id, indices.len());

            self.inner.append(vertices, device, queue);
            self.index.append(indices, device, queue);
        }
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
}
