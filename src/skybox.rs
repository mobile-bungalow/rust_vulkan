
use image::{ImageBuffer, ImageFormat, RgbaImage};

use crate::geometry::{Normal, Vertex};

pub struct SkyBox {
    textures: Vec<RgbaImage>,
    // vertices: [f32;32],
    // indices : [usize;36],
}

static TEXBYTES: [&[u8]; 6] = [
    include_bytes!("skybox/arrakisday_bk.tga"),
    include_bytes!("skybox/arrakisday_dn.tga"),
    include_bytes!("skybox/arrakisday_ft.tga"),
    include_bytes!("skybox/arrakisday_lf.tga"),
    include_bytes!("skybox/arrakisday_rt.tga"),
    include_bytes!("skybox/arrakisday_up.tga"),
];

impl SkyBox {
    pub fn new() -> Self {
        let textures: Vec<RgbaImage> = TEXBYTES
            .iter()
            .map(|tex| {
                image::load_from_memory_with_format(tex, ImageFormat::TGA)
                    .unwrap()
                    .to_rgba()
            })
            .collect();

        SkyBox { textures }
    }
}