use std::{
    any::TypeId,
    intrinsics::transmute,
    mem::size_of,
    sync::atomic::{AtomicU64, Ordering},
};

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use wgpu::{
    Buffer, BufferDescriptor, BufferUsage, Device, PresentMode, Queue, Surface, SwapChain,
    SwapChainDescriptor, SwapChainError, SwapChainFrame, Texture, TextureFormat, TextureUsage,
};

use crate::{graphics::pipelines, resources::get_image};
use pipelines::Pipeline;


pub fn get_aligned<T: Sized>(alignment: u64) -> u64 {
    alignment * (size_of::<T>() as u64 / alignment) + alignment
}
pub fn get_buffer_size<T: Sized>() -> u64 { get_aligned::<T>(wgpu::BIND_BUFFER_ALIGNMENT) }

pub struct Resources {
    pub textures:  DashMap<u64, Texture>,
    pub buffers:   DashMap<u64, Buffer>,
    pub pipelines: DashMap<TypeId, Box<dyn Pipeline>>,

    pub texture_cache: DashMap<String, u64>,

    pub buffer_counter:  AtomicU64,
    pub texture_counter: AtomicU64,
}
impl Resources {
    pub fn new() -> Self {
        Self {
            textures:  DashMap::new(),
            buffers:   DashMap::new(),
            pipelines: DashMap::new(),

            texture_cache: DashMap::new(),

            buffer_counter:  0.into(),
            texture_counter: 0.into(),
        }
    }
}

pub struct Renderer {
    pub device:    Device,
    pub queue:     Queue,
    pub surface:   Surface,
    pub swapchain: Option<SwapChain>,
    pub frame:     Option<SwapChainFrame>,

    resources: Resources,

    pub width:  u32,
    pub height: u32,
}

impl Renderer {
    pub fn new(device: Device, queue: Queue, surface: Surface) -> Self {
        Self {
            device,
            queue,
            surface,
            swapchain: None,
            frame: None,
            resources: Resources::new(),
            width: 1,
            height: 1,
        }
    }

    pub fn swap(&mut self) -> Result<(), SwapChainError> {
        if self.swapchain.is_some() {
            self.frame = Some(self.swapchain.as_ref().unwrap().get_current_frame()?);
        }
        Ok(())
    }

    pub fn present(&mut self) { self.frame = None; }

    pub fn create_swap_chain(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.swapchain = None;
            return;
        }
        self.swapchain = Some(
            self.device
                .create_swap_chain(&self.surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: TextureFormat::Bgra8Unorm,
                    width,
                    height,
                    present_mode: PresentMode::Mailbox,
                }),
        );
        println!("swapchain created with w: {} h: {}", width, height);
        self.width = width;
        self.height = height;
    }

    pub fn get_pipeline_mut<'s, P: Pipeline + 'static>(
        &'s self,
    ) -> RefMut<'s, TypeId, Box<dyn Pipeline>> {
        if !self.resources.pipelines.contains_key(&TypeId::of::<P>()) {
            self.resources
                .pipelines
                .insert(TypeId::of::<P>(), Box::new(P::new(&self)));
        }
        self.resources
            .pipelines
            .get_mut(&TypeId::of::<P>())
            .unwrap()
    }

    pub fn get_buffer<'a>(&'a self, id: u64) -> &'a Buffer {
        unsafe {
            transmute::<&Buffer, &'a Buffer>(self.resources.buffers.get(&id).unwrap().value())
        }
    }

    pub fn load_buffer<B: Sized + 'static>(
        &self, count: u64, usage: BufferUsage,
    ) -> Ref<u64, Buffer> {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            size: get_buffer_size::<B>() * count,
            usage,
            mapped_at_creation: false,
            label: Some(&format!("{:?}", TypeId::of::<B>())),
        });
        let id = self.resources.buffer_counter.fetch_add(1, Ordering::AcqRel);
        self.resources.buffers.insert(id, buffer);
        self.resources.buffers.get(&id).unwrap()
    }

    pub fn load_buffer_raw(&self, size: u64, usage: BufferUsage) -> Ref<u64, Buffer> {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            size,
            usage,
            mapped_at_creation: false,
            label: None,
        });
        let id = self.resources.buffer_counter.fetch_add(1, Ordering::AcqRel);
        self.resources.buffers.insert(id, buffer);
        self.resources.buffers.get(&id).unwrap()
    }

    pub fn unload_buffer(&self, id: u64) { self.resources.buffers.remove(&id).unwrap(); }

    pub fn get_texture<'a>(&'a self, id: u64) -> &'a Texture {
        unsafe {
            transmute::<&Texture, &'a Texture>(self.resources.textures.get(&id).unwrap().value())
        }
    }

    pub fn load_texture(&self, path: &str) -> Ref<u64, Texture> {
        let path = String::from(path);
        if self.resources.texture_cache.contains_key(&path) {
            let id = self.resources.texture_cache.get(&path).unwrap();
            if self.resources.textures.contains_key(id.value()) {
                return self.resources.textures.get(&id.value()).unwrap();
            }
        }

        let image = get_image(&path).unwrap().to_rgba8();
        let dimensions = image.dimensions();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size:            wgpu::Extent3d {
                width:  dimensions.0,
                height: dimensions.1,
                depth:  1,
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsage::all(),
            label:           Some(&path),
        });
        self.queue.write_texture(
            wgpu::TextureCopyView {
                texture:   &texture,
                mip_level: 0,
                origin:    wgpu::Origin3d::ZERO,
            },
            &image.into_raw(),
            wgpu::TextureDataLayout {
                offset:         0,
                bytes_per_row:  4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            wgpu::Extent3d {
                width:  dimensions.0,
                height: dimensions.1,
                depth:  1,
            },
        );

        let id = self
            .resources
            .texture_counter
            .fetch_add(1, Ordering::AcqRel);
        self.resources.texture_cache.insert(path, id);
        self.resources.textures.insert(id, texture);
        self.resources.textures.get(&id).unwrap()
    }
}
