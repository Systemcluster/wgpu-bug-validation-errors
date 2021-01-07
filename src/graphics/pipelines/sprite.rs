use std::{collections::HashMap, mem::size_of};

use bytemuck::bytes_of;
use itertools::Itertools;
use shipyard::{IntoFastIter, View};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindingResource, BlendDescriptor, BlendFactor, BlendOperation,
    BufferSize, BufferUsage, ColorStateDescriptor, ColorWrite, CullMode, FilterMode, FrontFace,
    IndexFormat, InputStepMode, PipelineLayoutDescriptor, PolygonMode, PrimitiveTopology,
    ProgrammableStageDescriptor, RasterizationStateDescriptor, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, TextureFormat,
    TextureViewDescriptor, VertexAttributeDescriptor, VertexBufferDescriptor, VertexFormat,
    VertexStateDescriptor,
};

use crate::{
    components::{Camera, CameraData, Sprite, Transform},
    graphics::{pipelines::Pipeline, renderer::Renderer},
    resources::get_shader,
};

pub struct SpritePipeline {
    pub bind_group_layout: BindGroupLayout,

    // texture_id -> bind_group
    pub bind_groups: HashMap<u64, BindGroup>,

    pub pipeline: RenderPipeline,

    pub camera_buffer: u64,

    pub vertex_buffer:      u64,
    pub vertex_buffer_data: Vec<u8>,
    pub vertex_buffer_size: u64,

    pub texture_sampler: Sampler,

    // vbuf_start, vbuf_length, texture_id
    pub draw_instances: Vec<(u32, u32, u64)>,
}
impl SpritePipeline {
    pub const STRIDE: u64 = 64;

    pub fn update_element_count(&mut self, renderer: &Renderer, count: u64) {
        let size = count * Self::STRIDE;
        if size > self.vertex_buffer_size {
            renderer.unload_buffer(self.vertex_buffer);

            while size > self.vertex_buffer_size {
                self.vertex_buffer_size *= 2;
            }

            self.vertex_buffer = *(renderer.load_buffer_raw(
                self.vertex_buffer_size,
                BufferUsage::VERTEX | BufferUsage::COPY_DST,
            ))
            .key();

            self.vertex_buffer_data.resize(size as usize, 0);
        }
    }

    pub fn prepare(
        &mut self, renderer: &Renderer, data: (&View<Transform>, &View<Sprite>), camera: &Camera,
    ) {
        let data_count = data.fast_iter().count() as u64;
        self.update_element_count(renderer, data_count);

        let vertex_buffer = renderer.get_buffer(self.vertex_buffer);
        let camera_buffer = renderer.get_buffer(self.camera_buffer);

        renderer
            .queue
            .write_buffer(&camera_buffer, 0, bytes_of(&camera.data()));

        self.vertex_buffer_data.clear();
        self.draw_instances.clear();

        data.fast_iter()
            .sorted_by(|(_, a), (_, b)| a.texture.cmp(&b.texture))
            .enumerate()
            .for_each(|(i, (transform, sprite))| {
                self.vertex_buffer_data
                    .extend_from_slice(bytes_of(transform));
                self.vertex_buffer_data
                    .extend_from_slice(bytes_of(sprite.data()));

                #[allow(clippy::map_entry)]
                if !self.bind_groups.contains_key(&sprite.texture) {
                    let texture = renderer.get_texture(sprite.texture);
                    let texture_view = texture.create_view(&TextureViewDescriptor::default());
                    let bind_group = renderer.device.create_bind_group(&BindGroupDescriptor {
                        layout:  &self.bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding:  0,
                                resource: BindingResource::Buffer {
                                    buffer: &camera_buffer,
                                    offset: 0,
                                    size:   BufferSize::new(size_of::<CameraData>() as u64),
                                },
                            },
                            BindGroupEntry {
                                binding:  1,
                                resource: BindingResource::TextureView(&texture_view),
                            },
                            BindGroupEntry {
                                binding:  2,
                                resource: BindingResource::Sampler(&self.texture_sampler),
                            },
                        ],
                        label:   None,
                    });
                    self.bind_groups.insert(sprite.texture, bind_group);
                }

                let len = self.draw_instances.len();
                if len == 0 {
                    self.draw_instances.push((0, 0, sprite.texture));
                } else if self.draw_instances[len - 1].2 != sprite.texture {
                    self.draw_instances[len - 1].1 = i as u32 - 1;
                    self.draw_instances
                        .push((i as u32, i as u32, sprite.texture));
                }
            });
        let len = self.draw_instances.len();
        self.draw_instances[len - 1].1 = data_count as u32 - 1;

        renderer
            .queue
            .write_buffer(&vertex_buffer, 0, &self.vertex_buffer_data[..]);
    }

    pub fn draw<'s>(&'s mut self, renderer: &'s Renderer, pass: &mut RenderPass<'s>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, (renderer.get_buffer(self.vertex_buffer)).slice(..));

        for (start, end, bind_group) in self.draw_instances.iter() {
            pass.set_bind_group(0, &self.bind_groups[&bind_group], &[]);
            pass.draw(0..6, *start..*end as u32);
        }
    }
}
impl Pipeline for SpritePipeline {
    fn new(renderer: &Renderer) -> Self {
        let bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding:    0,
                            visibility: wgpu::ShaderStage::VERTEX,
                            ty:         wgpu::BindingType::Buffer {
                                ty:                 wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size:   None,
                            },
                            count:      None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding:    1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty:         wgpu::BindingType::Texture {
                                multisampled:   false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count:      None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding:    2,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty:         wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering:  true,
                            },
                            count:      None,
                        },
                    ],
                    label:   None,
                });

        let vertex_buffer_size = 16;
        let vertex_buffer = renderer.load_buffer_raw(
            vertex_buffer_size,
            BufferUsage::VERTEX | BufferUsage::COPY_DST,
        );

        let camera_buffer =
            renderer.load_buffer::<CameraData>(1, BufferUsage::UNIFORM | BufferUsage::COPY_DST);

        let texture_sampler = renderer.device.create_sampler(&SamplerDescriptor {
            address_mode_u:   AddressMode::ClampToEdge,
            address_mode_v:   AddressMode::ClampToEdge,
            address_mode_w:   AddressMode::ClampToEdge,
            mag_filter:       FilterMode::Nearest,
            min_filter:       FilterMode::Linear,
            mipmap_filter:    FilterMode::Nearest,
            lod_min_clamp:    -100.0,
            lod_max_clamp:    100.0,
            compare:          None,
            anisotropy_clamp: None,
            border_color:     None,
            label:            None,
        });

        let pipeline_layout = renderer
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts:   &[&bind_group_layout],
                push_constant_ranges: &[],
                label:                None,
            });

        let vs_module = renderer
            .device
            .create_shader_module(&ShaderModuleDescriptor {
                source: wgpu::ShaderSource::SpirV(get_shader("sprite/simple.vert").unwrap().into()),
                label:  None,
                flags:  wgpu::ShaderFlags::empty(),
            });
        let fs_module = renderer
            .device
            .create_shader_module(&ShaderModuleDescriptor {
                source: wgpu::ShaderSource::SpirV(get_shader("sprite/simple.frag").unwrap().into()),
                label:  None,
                flags:  wgpu::ShaderFlags::empty(),
            });

        let pipeline = renderer
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                layout:                    Some(&pipeline_layout),
                vertex_stage:              ProgrammableStageDescriptor {
                    module:      &vs_module,
                    entry_point: "main",
                },
                fragment_stage:            Some(ProgrammableStageDescriptor {
                    module:      &fs_module,
                    entry_point: "main",
                }),
                rasterization_state:       Some(RasterizationStateDescriptor {
                    front_face:             FrontFace::Ccw,
                    cull_mode:              CullMode::None,
                    depth_bias:             0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp:       0.0,
                    clamp_depth:            false,
                    polygon_mode:           PolygonMode::Fill,
                }),
                primitive_topology:        PrimitiveTopology::TriangleList,
                color_states:              &[ColorStateDescriptor {
                    format:      TextureFormat::Bgra8Unorm,
                    color_blend: BlendDescriptor {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation:  BlendOperation::Add,
                    },
                    alpha_blend: BlendDescriptor {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation:  BlendOperation::Add,
                    },
                    write_mask:  ColorWrite::ALL,
                }],
                depth_stencil_state:       None,
                sample_count:              1,
                alpha_to_coverage_enabled: false,
                sample_mask:               0,
                vertex_state:              VertexStateDescriptor {
                    index_format:   None,
                    vertex_buffers: &[VertexBufferDescriptor {
                        stride:     Self::STRIDE,
                        step_mode:  InputStepMode::Instance,
                        attributes: &[
                            VertexAttributeDescriptor {
                                offset:          0,
                                shader_location: 0,
                                format:          VertexFormat::Float4,
                            },
                            VertexAttributeDescriptor {
                                offset:          16,
                                shader_location: 1,
                                format:          VertexFormat::Float4,
                            },
                            VertexAttributeDescriptor {
                                offset:          32,
                                shader_location: 2,
                                format:          VertexFormat::Float4,
                            },
                            VertexAttributeDescriptor {
                                offset:          48,
                                shader_location: 3,
                                format:          VertexFormat::Float2,
                            },
                            VertexAttributeDescriptor {
                                offset:          56,
                                shader_location: 4,
                                format:          VertexFormat::Float2,
                            },
                        ],
                    }],
                },
                label:                     None,
            });

        Self {
            bind_group_layout,

            bind_groups: HashMap::new(),

            pipeline,

            vertex_buffer: *vertex_buffer.key(),
            vertex_buffer_data: Vec::new(),
            vertex_buffer_size,

            camera_buffer: *camera_buffer.key(),

            texture_sampler,

            draw_instances: Vec::new(),
        }
    }
}
