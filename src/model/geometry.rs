use super::{
    transform::{Rotate, Scale, Translate},
    Expandable, ModelContext,
};

impl<T: Translate> Translate for [T] {
    fn translate(&mut self, translation: glam::Vec3) {
        for item in self.iter_mut() {
            item.translate(translation);
        }
    }
}

impl<T: Rotate> Rotate for [T] {
    fn rotate(&mut self, rotation: glam::Quat) {
        for item in self.iter_mut() {
            item.rotate(rotation);
        }
    }
}

impl<T: Scale> Scale for [T] {
    fn scale(&mut self, scale: glam::Vec3) {
        for item in self.iter_mut() {
            item.scale(scale);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Geometry<T> {
    Simple { vertices: Vec<T> },
    Indexed { indices: Vec<u32>, vertices: Vec<T> },
}

impl<T> Geometry<T> {
    pub fn empty() -> Self {
        Self::Simple {
            vertices: Vec::new(),
        }
    }

    pub fn indexed_empty() -> Self {
        Self::Indexed {
            indices: Vec::new(),
            vertices: Vec::new(),
        }
    }

    pub fn vertices(&self) -> &[T] {
        match self {
            Self::Simple { vertices } => vertices,
            Self::Indexed { vertices, .. } => vertices,
        }
    }

    pub fn indices(&self) -> Option<&[u32]> {
        match self {
            Self::Simple { .. } => None,
            Self::Indexed { indices, .. } => Some(indices),
        }
    }

    pub fn push(&mut self, vertex: T) {
        match self {
            Self::Simple { vertices } => vertices.push(vertex),
            Self::Indexed { vertices, .. } => vertices.push(vertex),
        }
    }

    pub fn push_index(&mut self, index: u32) {
        match self {
            Self::Simple { .. } => {}
            Self::Indexed { indices, .. } => indices.push(index),
        }
    }
}

impl<T: Clone> ModelContext for Geometry<T> {}

impl<T: Clone> Expandable for Geometry<T> {
    fn expand(&mut self, other: &Self) {
        match self {
            Self::Simple { vertices } => {
                if let Self::Simple {
                    vertices: other_vertices,
                } = other
                {
                    vertices.extend_from_slice(other_vertices);
                }
            }
            Self::Indexed { indices, vertices } => {
                if let Self::Indexed {
                    indices: other_indices,
                    vertices: other_vertices,
                } = other
                {
                    let offset = vertices.len() as u32;
                    indices.extend(other_indices.iter().map(|i| i + offset));
                    vertices.extend_from_slice(other_vertices);
                }
            }
        }
    }
}

impl<T: Translate> Translate for Geometry<T> {
    fn translate(&mut self, translation: glam::Vec3) {
        match self {
            Self::Simple { vertices } => vertices.translate(translation),
            Self::Indexed { vertices, .. } => vertices.translate(translation),
        }
    }
}

impl<T: Rotate> Rotate for Geometry<T> {
    fn rotate(&mut self, rotation: glam::Quat) {
        match self {
            Self::Simple { vertices } => vertices.rotate(rotation),
            Self::Indexed { vertices, .. } => vertices.rotate(rotation),
        }
    }
}

impl<T: Scale> Scale for Geometry<T> {
    fn scale(&mut self, scale: glam::Vec3) {
        match self {
            Self::Simple { vertices } => vertices.scale(scale),
            Self::Indexed { vertices, .. } => vertices.scale(scale),
        }
    }
}
