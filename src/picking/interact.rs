use glam::Vec2;
use winit::event::MouseButton;

pub trait Interactive {
    fn mouse_clicked(&mut self, button: MouseButton) {}
    fn mouse_scroll(&mut self, delta: f32) {}
    fn mouse_motion(&mut self, button: MouseButton, delta: Vec2) {}
}
