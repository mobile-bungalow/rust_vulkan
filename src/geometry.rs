use vulkano::impl_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32, f32),
}

impl_vertex!(Vertex, position);

/// TODO: replace these structs with
/// the built in CG math library.
/// this seems like a no brainer
/// also implement a 'faces' trait,
/// which returns an iterator referencing
/// faces based off an index string.
#[derive(Copy, Clone)]
pub struct Normal {
    pub normal: (f32, f32, f32),
}
/// same as other do to
impl_vertex!(Normal, normal);

// static SBIDIM: f32 = 100.0;

// // skybox vertices
// pub static SB: [Vertex; 32] = {
//     [Vertex {
//         position: (0.0, 0.0, 0.0),
//     }; 32]
// };

// // skybox indices
// pub static SBI: [usize; 32] = { [0; 32] };

// TODO: skybox textures

/// yeah, the name is long, fight me.
pub fn norms_from_verts_and_index(model_verts: &[Vertex], indices: &[u32]) -> Vec<Normal> {
    let mut model_normals: Vec<Normal> = vec![
        Normal {
            normal: (0.0, 0.0, 0.0),
        };
        model_verts.len()
    ];

    indices
        .chunks(3)
        .map(|a| ((a[0]) as usize, a[1] as usize, a[2] as usize))
        .for_each(|(i_a, i_b, i_c)| {
            // the indices into the model vertex triples vector
            // which indicates one vector of a face

            //get the vectors that compose the face,
            // as a note, it would save us a lot of clone
            // space (even thought this is a one off) to do
            // just store vectors as cgmath::Vector3 if possible. QOL
            let v_a = {
                let i = model_verts[i_a];
                cgmath::Vector3::new(i.position.0, i.position.1, i.position.2)
            };

            let v_b = {
                let i = model_verts[i_b];
                cgmath::Vector3::new(i.position.0, i.position.1, i.position.2)
            };

            let v_c = {
                let i = model_verts[i_c];
                cgmath::Vector3::new(i.position.0, i.position.1, i.position.2)
            };

            //compute the cross product
            let cross = (v_b - v_a).cross(v_c - v_b);

            //map to running sum
            model_normals[i_a].normal.0 += cross.x;
            model_normals[i_a].normal.1 += cross.y;
            model_normals[i_a].normal.2 += cross.z;

            model_normals[i_b].normal.0 += cross.x;
            model_normals[i_b].normal.1 += cross.y;
            model_normals[i_b].normal.2 += cross.z;

            model_normals[i_c].normal.0 += cross.x;
            model_normals[i_c].normal.1 += cross.y;
            model_normals[i_c].normal.2 += cross.z;
        });

    // case and point for moving everything to cgmath
    model_normals
        .iter()
        .map(|x| (x.normal.0, x.normal.1, x.normal.2))
        .for_each(|mut v| {
            let mut vec = cgmath::Vector3::new(v.0, v.1, v.2);
            // v /= |v| i.e. normalize
            vec /= (vec.x.powf(2.0) + vec.y.powf(2.0) + vec.z.powf(2.0)).sqrt();
            v.0 = vec.x;
            v.1 = vec.y;
            v.2 = vec.z;
        });
    model_normals
}
