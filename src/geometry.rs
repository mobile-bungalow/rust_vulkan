
use std::ops;
use vulkano::impl_vertex;
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32, f32),
}

impl_vertex!(Vertex, position);

impl ops::Add<Vertex> for Vertex {
    type Output = Vertex;
    fn add(self, rhs: Vertex) -> Vertex {
        Vertex {
            position: (
                self.position.0 + rhs.position.0,
                self.position.1 + rhs.position.1,
                self.position.2 + rhs.position.2,
            ),
        }
    }
}

impl ops::Div<f32> for Vertex {
    type Output = Vertex;
    fn div(self, rhs: f32) -> Vertex {
        Vertex {
            position: (
                self.position.0 / rhs,
                self.position.1 / rhs,
                self.position.2 / rhs,
            ),
        }
    }
}

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
// same as other do to
impl_vertex!(Normal, normal);


// TODO: skybox textures

/// yeah, the name is long, fight me.
pub fn norms_from_verts_and_index(
    model_verts: &[Vertex],
    indices: &[u32],
) -> (Vec<Normal>, [Vertex; 2]) {
    let mut model_normals: Vec<Normal> = vec![
        Normal {
            normal: (0.0, 0.0, 0.0),
        };
        model_verts.len()
    ];

    let mut min_verts = Vertex {
        position: (std::f32::MAX, std::f32::MAX, std::f32::MAX),
    };

    let mut max_verts = Vertex {
        position: (std::f32::MIN, std::f32::MIN, std::f32::MIN),
    };

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

            [v_a, v_b, v_c]
                .iter()
                .map(|vec| (vec.x, vec.y, vec.z))
                .for_each(|(x, y, z)| {

                    if x < min_verts.position.0 {
                        min_verts.position.0 = x
                    }

                    if x > max_verts.position.0 {
                        max_verts.position.0 = x
                    }


                    if y < min_verts.position.1 {
                        min_verts.position.1 = y
                    }

                    if y > max_verts.position.1 {
                        max_verts.position.1 = y
                    }


                    if z < min_verts.position.2 {
                        min_verts.position.2 = z
                    }

                    if z > max_verts.position.2 {
                        max_verts.position.2 = z
                    }

                });

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
    (model_normals, [max_verts, min_verts])
}
