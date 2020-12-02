use std::{env, path, path::Path};

use image::{DynamicImage, ImageResult};

use crate::shaders;


pub fn get_path<P: AsRef<Path>>(path: P) -> path::PathBuf {
    env::current_dir().unwrap().join("data").join(&path)
}

pub fn get_image<P: AsRef<Path>>(path: P) -> ImageResult<DynamicImage> {
    image::open(get_path("images").join(&path))
}

pub fn get_shader<P: AsRef<Path>>(path: P) -> Result<Vec<u32>, shaderc::Error> {
    shaders::get_shader(&path)
}
