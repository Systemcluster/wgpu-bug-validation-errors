#![allow(clippy::identity_op)]

use bytemuck::{Pod, Zeroable};
use ultraviolet::{projection::perspective_wgpu_dx, Mat4, Vec2, Vec3, Vec4};

#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct Transform {
    pub position: Vec4,
    pub rotation: Vec4,
    pub size:     Vec4,
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub eye:    Vec3,
    pub target: Vec3,
    pub up:     Vec3,
    pub aspect: f32,
    pub fovy:   f32,
    pub znear:  f32,
    pub zfar:   f32,
}
impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Vec3::new(0.0, 0.0, 0.0),
            target: Vec3::new(0.0, 0.0, 100.0),
            up: Vec3::unit_y(),
            aspect,
            fovy: 90.0,
            znear: 0.0,
            zfar: 100.0,
        }
    }

    pub fn projection(&self) -> Mat4 {
        perspective_wgpu_dx(self.fovy.to_radians(), self.aspect, self.znear, self.zfar)
    }

    pub fn view(&self) -> Mat4 { Mat4::look_at(self.eye, self.target, self.up) }

    pub fn data(&self) -> CameraData {
        CameraData {
            projection: self.projection() * self.view(),
        }
    }
}
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct CameraData {
    pub projection: Mat4,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Sprite {
    pub data:    SpriteData,
    pub texture: u64,
}
impl Sprite {
    pub fn data(&self) -> &SpriteData { &self.data }
}
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct SpriteData {
    pub texture_position: Vec2,
    pub texture_size:     Vec2,
}
