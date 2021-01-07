use shipyard::{UniqueView, UniqueViewMut, View};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachmentDescriptor,
    RenderPassDescriptor,
};

use crate::{
    components::{Camera, Sprite, Transform},
    graphics::{pipelines::*, renderer::Renderer},
};


pub fn render(
    camera: UniqueView<Camera>, renderer: UniqueViewMut<Renderer>, transforms: View<Transform>,
    sprites: View<Sprite>,
) {
    if renderer.frame.is_none() {
        return;
    }
    let view = &renderer.frame.as_ref().unwrap().output.view;

    let mut pipeline = renderer.get_pipeline_mut::<SpritePipeline>();
    let pipeline = pipeline.downcast_mut::<SpritePipeline>();

    pipeline.prepare(&renderer, (&transforms, &sprites), &camera);

    let mut encoder = renderer
        .device
        .create_command_encoder(&CommandEncoderDescriptor { label: None });
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments:        &[RenderPassColorAttachmentDescriptor {
                attachment:     &view,
                resolve_target: None,
                ops:            Operations {
                    load:  LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
            label:                    None,
        });
        pipeline.draw(&renderer, &mut pass);
    }
    renderer.queue.submit(Some(encoder.finish()));
}
