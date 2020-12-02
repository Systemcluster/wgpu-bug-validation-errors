use rand::random;
use shipyard::{system, UniqueViewMut, Workload, World};
use ultraviolet::{Vec2, Vec4};
use wgpu::{Device, Queue, Surface};
use winit::event::WindowEvent;

use crate::{
    components::{Camera, Sprite, SpriteData, Transform},
    graphics::renderer::Renderer,
    systems::*,
};


pub struct Universe {
    pub world: World,
}

impl Universe {
    pub fn new(device: Device, queue: Queue, surface: Surface) -> Self {
        let mut world = World::new();

        world.add_unique(Camera::new(1.0));

        let renderer = Renderer::new(device, queue, surface);

        let textures = [
            "monochrome_transparent_packed.png",
            "colored_transparent_packed.png",
        ];
        let elements = 200;
        for x in 0..elements {
            for y in 0..elements {
                world.add_entity((
                    Transform {
                        position: Vec4::new(
                            (-1.0 + ((x as f32 + 0.5) / (elements as f32 / 2.0))) * 10.0,
                            (-1.0 + ((y as f32 + 0.5) / (elements as f32 / 2.0))) * 10.0,
                            5.0,
                            1.0,
                        ),
                        rotation: Vec4::zero(),
                        size:     Vec4::new(5.0 / elements as f32, 5.0 / elements as f32, 0.5, 1.0),
                    },
                    Sprite {
                        texture: *renderer
                            .load_texture(textures[rand::random::<usize>() % 2])
                            .key(),
                        data:    SpriteData {
                            texture_position: Vec2::new(
                                (random::<f32>() * 48.0).round() / 48.0,
                                (random::<f32>() * 22.0).round() / 22.0,
                            ),
                            texture_size:     Vec2::new(1.0 / 48.0, 1.0 / 22.0),
                        },
                    },
                ));
            }
        }

        world.add_unique(renderer);

        Workload::builder("main")
            .with_system(system!(render))
            .add_to_world(&world)
            .unwrap();

        Self { world }
    }

    pub fn render(&mut self) {
        self.world.run(|mut renderer: UniqueViewMut<Renderer>| {
            renderer.swap().unwrap();
        });
        self.world.run_workload("main");
        self.world.run(|mut renderer: UniqueViewMut<Renderer>| {
            renderer.present();
        });
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.world.run(
            |mut renderer: UniqueViewMut<Renderer>, mut camera: UniqueViewMut<Camera>| {
                camera.aspect = width as f32 / height as f32;
                renderer.create_swap_chain(width, height);
            },
        );
    }

    pub fn event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(new_inner_size) => {
                self.resize(new_inner_size.width, new_inner_size.height);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(new_inner_size.width, new_inner_size.height);
            }
            _ => (),
        };
    }
}
