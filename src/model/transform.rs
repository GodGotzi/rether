use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

pub trait Translate {
    fn translate(&mut self, translation: glam::Vec3);
}

pub trait Rotate {
    fn rotate(&mut self, rotation: glam::Quat, center: Vec3);
}

pub trait Scale {
    fn scale(&mut self, scale: glam::Vec3);
}

impl Translate for Transform {
    fn translate(&mut self, translation: glam::Vec3) {
        self.translation += translation;
    }
}

impl Rotate for Transform {
    fn rotate(&mut self, rotation: glam::Quat, _center: Vec3) {
        self.rotation = rotation * self.rotation;
    }
}

impl Scale for Transform {
    fn scale(&mut self, scale: glam::Vec3) {
        self.scale *= scale;
    }
}

impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}
