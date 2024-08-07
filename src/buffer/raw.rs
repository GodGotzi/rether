use std::cell::OnceCell;

use super::alloc::ModifyAction;
use wgpu::{BufferAddress, BufferDescriptor};

#[derive(Debug)]
pub struct RawBuffer {
    pub inner: wgpu::Buffer,
    pub render_range: std::ops::Range<u32>,

    usage: wgpu::BufferUsages,

    pub size: BufferAddress,
    label: String,
}

impl RawBuffer {
    pub fn new<T>(
        size: usize,
        label: &str,
        usage: wgpu::BufferUsages,
        device: &wgpu::Device,
    ) -> Self
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

    pub fn allocate<T>(&mut self, size: usize, device: &wgpu::Device, queue: &wgpu::Queue)
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

    pub fn append<T>(&mut self, data: &[T], device: &wgpu::Device, queue: &wgpu::Queue)
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

    pub fn free<T>(
        &mut self,
        offset: usize,
        size: usize,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) where
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

    pub fn write<T>(&self, queue: &wgpu::Queue, offset: usize, data: &[T])
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let offset_bytes = offset * std::mem::size_of::<T>();

        queue.write_buffer(&self.inner, offset_bytes as u64, bytemuck::cast_slice(data));
    }

    pub fn modify<T>(
        &self,
        mut modify_action: ModifyAction<T>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let offset_bytes = modify_action.offset * std::mem::size_of::<T>();
        let size_bytes = modify_action.size * std::mem::size_of::<T>();

        let read_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Read Buffer"),
            size: size_bytes as BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: true,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &self.inner,
            offset_bytes as BufferAddress,
            &read_buffer,
            0,
            size_bytes as BufferAddress,
        );

        queue.submit(std::iter::once(encoder.finish()));

        let once_cell = OnceCell::new();

        let cloned_once_cell = once_cell.clone();

        read_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                cloned_once_cell.set(result).unwrap();
            });
        device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = once_cell.get() {
            let raw_data = read_buffer.slice(..).get_mapped_range();

            let mut data = bytemuck::cast_slice::<u8, T>(&raw_data).to_vec();
            modify_action.act(&mut data);

            read_buffer.unmap();
            read_buffer.destroy();

            self.write(queue, modify_action.offset, &data);
        }
    }
}
