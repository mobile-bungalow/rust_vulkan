use image::{ImageFormat, RgbaImage};
use crate::geometry::Vertex;


pub struct SkyBox {
    pub textures: Vec<RgbaImage>,
    pub vertices: [Vertex; 24],
    pub indices: [usize; 36],
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

        let vertices = [
            // Front
            Vertex {
                position: (-2.0, -2.0, 2.0),
            },
            Vertex {
                position: (2.0, -2.0, 2.0),
            },
            Vertex {
                position: (2.0, 2.0, 2.0),
            },
            Vertex {
                position: (-2.0, 2.0, 2.0),
            },
            // Right
            Vertex {
                position: (2.0, -2.0, 2.0),
            },
            Vertex {
                position: (2.0, -2.0, -2.0),
            },
            Vertex {
                position: (2.0, 2.0, -2.0),
            },
            Vertex {
                position: (2.0, 2.0, 2.0),
            },
            // Back
            Vertex {
                position: (-2.0, -2.0, -2.0),
            },
            Vertex {
                position: (-2.0, 2.0, -2.0),
            },
            Vertex {
                position: (2.0, 2.0, -2.0),
            },
            Vertex {
                position: (2.0, -2.0, -2.0),
            },
            // Left
            Vertex {
                position: (-2.0, -2.0, 2.0),
            },
            Vertex {
                position: (-2.0, 2.0, 2.0),
            },
            Vertex {
                position: (-2.0, 2.0, -2.0),
            },
            Vertex {
                position: (-2.0, -2.0, -2.0),
            },
            // Bottom
            Vertex {
                position: (-2.0, -2.0, 2.0),
            },
            Vertex {
                position: (-2.0, -2.0, -2.0),
            },
            Vertex {
                position: (2.0, -2.0, -2.0),
            },
            Vertex {
                position: (2.0, -2.0, 2.0),
            },
            // Top
            Vertex {
                position: (-2.0, 2.0, 2.0),
            },
            Vertex {
                position: (2.0, 2.0, 2.0),
            },
            Vertex {
                position: (2.0, 2.0, -2.0),
            },
            Vertex {
                position: (-2.0, 2.0, -2.0),
            },
        ];

        let indices = [
            0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16,
            17, 18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
        ];

        SkyBox {
            textures,
            vertices,
            indices,
        }
    }
}