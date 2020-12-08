use crate::graphics::renderer::Renderer;

pub trait Pipeline: Send + Sync {
    fn new(renderer: &Renderer) -> Self
    where
        Self: Sized;
}
impl dyn Pipeline {
    pub fn downcast_mut<P: Pipeline>(&mut self) -> &mut P {
        unsafe { &mut *(self as *mut dyn Pipeline as *mut P) }
    }
}

pub mod sprite;
pub use sprite::*;
