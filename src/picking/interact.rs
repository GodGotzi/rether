use glam::Vec2;
use winit::{event::MouseButton, keyboard::KeyCode};

pub enum Action {
    Mouse(MouseButton),
    Keyboard(KeyCode),
}

pub struct DragEvent {
    pub delta: Vec2,
    pub action: Action,
}

pub struct ClickEvent {
    pub action: Action,
}

pub struct ScrollEvent {
    pub delta: f32,
    pub action: Action,
}

pub trait Interactive {
    fn clicked(&mut self, event: ClickEvent);
    fn scroll(&mut self, event: ScrollEvent);
    fn drag(&mut self, event: DragEvent);
}

pub trait InteractiveModel {
    fn clicked(&self, event: ClickEvent);
    fn drag(&self, event: DragEvent);
    fn scroll(&self, event: ScrollEvent);
}
