use vulkano::impl_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32, f32),
}

impl_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub struct Normal {
    pub normal: (f32, f32, f32),
}

impl_vertex!(Normal, normal);

static sb_dim: f32 = 100.0;

// skybox vertices
pub static sb: [Vertex; 32] = {
    [Vertex {
        position: (0.0, 0.0, 0.0),
    }; 32]
};

// skybox indices
pub static sbi: [usize; 32] = { [0; 32] };

// TODO: skybox textures
