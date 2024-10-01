use glam::Vec3;

use crate::{
    buffer::{BufferData, IndexedBufferData},
    vertex::{Vertex, VertexRotator},
};

use super::{
    transform::{Rotate, Scale, Translate},
    Expandable,
};

impl<T: Translate> Translate for [T] {
    fn translate(&mut self, translation: glam::Vec3) {
        for item in self.iter_mut() {
            item.translate(translation);
        }
    }
}

impl Rotate for [Vertex] {
    fn rotate(&mut self, rotation: glam::Quat, center: Vec3) {
        VertexRotator::new(self).rotate(rotation, center);
    }
}

impl<T: Scale> Scale for [T] {
    fn scale(&mut self, scale: glam::Vec3) {
        for item in self.iter_mut() {
            item.scale(scale);
        }
    }
}

pub trait Geometry: Expandable {
    type Data<'a>
    where
        Self: 'a;

    fn build_data(&self) -> Self::Data<'_>;
    fn data_len(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct SimpleGeometry<T> {
    vertices: Vec<T>,
}

impl<T> SimpleGeometry<T> {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }

    pub fn init(vertices: Vec<T>) -> Self {
        Self { vertices }
    }
}

impl<T> Geometry for SimpleGeometry<T>
where
    T: Clone,
{
    type Data<'a> = BufferData<'a, T> where T: 'a;

    fn build_data(&self) -> Self::Data<'_> {
        BufferData::create(&self.vertices)
    }

    fn data_len(&self) -> usize {
        self.vertices.len()
    }
}

impl<T: Clone> Expandable for SimpleGeometry<T> {
    fn expand(&mut self, other: &Self) {
        self.vertices.extend_from_slice(&other.vertices);
    }
}

impl<T: Translate> Translate for SimpleGeometry<T> {
    fn translate(&mut self, translation: glam::Vec3) {
        self.vertices.translate(translation)
    }
}

impl Rotate for SimpleGeometry<Vertex> {
    fn rotate(&mut self, rotation: glam::Quat, center: Vec3) {
        self.vertices.rotate(rotation, center)
    }
}

impl<T: Scale> Scale for SimpleGeometry<T> {
    fn scale(&mut self, scale: glam::Vec3) {
        self.vertices.scale(scale)
    }
}

#[derive(Debug, Clone)]
pub struct IndexedGeometry<T> {
    vertices: Vec<T>,
    indices: Vec<u32>,
}

impl<T: Clone> IndexedGeometry<T> {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn init(vertices: Vec<T>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn into_simple(self) -> SimpleGeometry<T> {
        SimpleGeometry::init(self.vertices)
    }
}

impl<T> Geometry for IndexedGeometry<T>
where
    T: Clone,
{
    type Data<'a> = IndexedBufferData<'a, T> where T: 'a;

    fn build_data(&self) -> Self::Data<'_> {
        IndexedBufferData::create(&self.vertices, &self.indices)
    }

    fn data_len(&self) -> usize {
        self.vertices.len()
    }
}

impl<T: Clone> Expandable for IndexedGeometry<T> {
    fn expand(&mut self, other: &Self) {
        self.vertices.extend_from_slice(&other.vertices);

        let offset = self.vertices.len() as u32;

        let indices = other
            .indices
            .iter()
            .map(|index| *index + offset)
            .collect::<Vec<u32>>();

        self.indices.extend_from_slice(&indices);
    }
}

impl<T: Translate> Translate for IndexedGeometry<T> {
    fn translate(&mut self, translation: glam::Vec3) {
        self.vertices.translate(translation)
    }
}

impl Rotate for IndexedGeometry<Vertex> {
    fn rotate(&mut self, rotation: glam::Quat, center: Vec3) {
        self.vertices.rotate(rotation, center)
    }
}

impl<T: Scale> Scale for IndexedGeometry<T> {
    fn scale(&mut self, scale: glam::Vec3) {
        self.vertices.scale(scale)
    }
}
