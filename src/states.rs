use crate::universe::Universe;

use winit::event::Event;


pub trait State {
    fn new(universe: &Universe) -> Self
    where
        Self: Sized;
    fn init(&mut self, universe: &Universe);
    fn event(&mut self, universe: &Universe, event: Event<()>);
    fn update(&mut self, universe: &Universe);
}

pub struct EmptyState {}
impl State for EmptyState {
    fn new(_universe: &Universe) -> Self { Self {} }

    fn init(&mut self, _universe: &Universe) {}

    fn event(&mut self, _universe: &Universe, _event: Event<()>) {}

    fn update(&mut self, _universe: &Universe) {}
}
