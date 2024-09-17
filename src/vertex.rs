/// A vertex is a single point. A geometry is typically composed of multiple vertecies.
use bytemuck::Zeroable;

use crate::{model::transform::Translate, Rotate, Scale};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Translate for Vertex {
    fn translate(&mut self, translation: glam::Vec3) {
        self.position[0] += translation.x;
        self.position[1] += translation.y;
        self.position[2] += translation.z;
    }
}

impl Rotate for Vertex {
    fn rotate(&mut self, rotation: glam::Quat) {
        let position = glam::Vec3::from(self.position);
        let normal = glam::Vec3::from(self.normal);

        self.position = (rotation * position).into();
        self.normal = (rotation * normal).into();
    }
}

impl Scale for Vertex {
    fn scale(&mut self, scale: glam::Vec3) {
        self.position[0] *= scale.x;
        self.position[1] *= scale.y;
        self.position[2] *= scale.z;
    }
}

pub struct VertexRotator<'a, T> {
    data: &'a mut [T],
    center: glam::Vec3,
}

impl<'a, T> VertexRotator<'a, T> {
    pub fn new(data: &'a mut [T], center: glam::Vec3) -> Self {
        Self { data, center }
    }
}

impl<'a> Rotate for VertexRotator<'a, Vertex> {
    fn rotate(&mut self, rotation: glam::Quat) {
        for vertex in self.data.iter_mut() {
            let position = glam::Vec3::from(vertex.position);
            let normal = glam::Vec3::from(vertex.normal);

            vertex.position = (rotation * (position - self.center) + self.center).into();
            vertex.normal = (rotation * normal).into();
        }
    }
}

pub struct VertexScaler<'a, T> {
    data: &'a mut [T],
    center: glam::Vec3,
}

impl<'a, T> VertexScaler<'a, T> {
    pub fn new(data: &'a mut [T], center: glam::Vec3) -> Self {
        Self { data, center }
    }
}

impl<'a> Scale for VertexScaler<'a, Vertex> {
    fn scale(&mut self, scale: glam::Vec3) {
        for vertex in self.data.iter_mut() {
            let position = glam::Vec3::from(vertex.position);

            vertex.position = ((position - self.center) * scale + self.center).into();
        }
    }
}
