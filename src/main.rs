/// Imports for loading files
use std::path::Path;

use std::iter;

/// types from the object parser
use tobj;

/// This is a geometry, should be removed once the obj parser is up and running!
mod geometry;

/// This loads textures the skybox textures and indices
mod skybox;

/// Vulkan imports, these are manifold , low level, and sinful.
use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use image::ImageFormat;
use skybox::SkyBox;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::{Dimensions, ImmutableImage, SwapchainImage};
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, SwapchainCreationError};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use winit::Window;

use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};

use std::sync::Arc;

/// arg parse
use clap::{App, Arg};

mod vk;

static DAMPENING: f32 = 0.01;

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
// from the teapot example 
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 v_normal;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    mat4 translate;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    gl_Position = uniforms.proj * worldview * (uniforms.translate * vec4(position, 1.0)) ;
}

",
    }
}

// fragment shader
mod fs {
    vulkano_shaders::shader! {
    ty: "fragment",
        src: "
// from the teapot example 

#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 0) out vec4 f_color;

const vec3 LIGHT = vec3(0.0, 0.0, 1.0);

void main() {
    float brightness = dot(normalize(v_normal), normalize(LIGHT));
    vec3 dark_color = vec3(0.6, 0.0, 0.0);
    vec3 regular_color = vec3(1.0, 0.0, 0.0);

    f_color = vec4(mix(dark_color, regular_color, brightness), 1.0);
}
        ",
    }
}

fn main() {
    // arg parsing, fails the program without input file
    let matches = App::new("vk_obj")
        .version("1.0")
        .author("Paul May, pwmay@ucsc.edu")
        .about("cross platform Obj loader in rust using vulkan bindings")
        .arg(
            Arg::with_name("input")
                .required(true)
                .short("i")
                .long("input")
                .value_name("fname")
                .help("the name of the input wavefront .obj file")
                .takes_value(true),
        )
        .get_matches();

    // parse object with tiny object loader
    let obj_file = tobj::load_obj(&Path::new(matches.value_of("input").unwrap()));
    assert!(obj_file.is_ok());

    let (geom, _mats) = obj_file.unwrap();

    let skybox = SkyBox::new();

    let image_data = skybox.textures.iter().cloned().map(|x| x.into_raw()).fold(
        Vec::new(),
        |mut agg: Vec<u8>, mut x| {
            agg.append(&mut x);
            agg
        },
    );

    // break the positions up into groups of three
    let model_verts: Vec<geometry::Vertex> = geom[0]
        .mesh
        .positions
        .chunks(3) //breaks into chunks of threes
        .map(|chunk| geometry::Vertex {
            position: (chunk[0], chunk[1], chunk[2]),
        })
        .collect();


    // generate the normals for the model at each vertex
    let (model_normals, extent) =
        geometry::norms_from_verts_and_index(&model_verts, &geom[0].mesh.indices);

    let mut vk_state: vk::VKState = vk::VKState::vk_init().expect("initialization failed \n");
    let window = vk_state.surface.window();

    let (texture, tex_future) = {
        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Cubemap { size: 512 },
            Format::R8G8B8A8Srgb,
            vk_state.queue.clone(),
        )
        .unwrap()
    };

    let sampler = Sampler::new(
        vk_state.device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap();

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        vk_state.device.clone(),
        BufferUsage::all(),
        model_verts.iter().cloned(),
    )
    .unwrap();

    let normals_buffer = CpuAccessibleBuffer::from_iter(
        vk_state.device.clone(),
        BufferUsage::all(),
        model_normals.iter().cloned(),
    )
    .unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(
        vk_state.device.clone(),
        BufferUsage::all(),
        geom[0].mesh.indices.iter().cloned(),
    )
    .unwrap();

    let uniform_buffer =
        CpuBufferPool::<vs::ty::Data>::new(vk_state.device.clone(), BufferUsage::all());

    // compile frag and vertex shaders here
    let vs = vs::Shader::load(vk_state.device.clone()).unwrap();
    let fs = fs::Shader::load(vk_state.device.clone()).unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(vk_state.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: vk_state.swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap(),
    );

    // todo: figure out what the hell these are for
    // they are an abstract part of this boilerplate
    let (mut pipeline, mut framebuffers) = window_size_dependent_setup(
        vk_state.device.clone(),
        &vs,
        &fs,
        &vk_state.images,
        render_pass.clone(),
    );

    let mut recreate_swapchain = false;
    let mut previous_frame = Box::new(sync::now(vk_state.device.clone())) as Box<GpuFuture>;

    // these are used to rotate the world projection
    // modifed in the mouse events after each frame,
    // used in the uniform sub buffer each frame
    let mut y_delta: f32 = 0.0;
    let mut x_delta: f32 = 0.0;
    let mut mouse_state: winit::ElementState = winit::ElementState::Released;

    // translation matrix
    // which moves the model to the origin.
    let offset = extent[0] + extent[1];
    let (x, y, z) = (offset / 2.0).position;
    let translate = Matrix4::from_translation(Vector3::new(-x, -y, -z));
    // scale everything s.t. it never exceeds three units in any direction.
    let (i, j, k) = extent[0].position;
    let (l, m, n) = extent[1].position;
    let mut max = std::f32::MIN;

    for val in [i, j, k, l, m, n].iter().map(|x| x.abs()) {
        max = if val > max { val } else { max };
    }
    // scale is whatever reduces the largest
    // dimension to 0.3, because that is a nice size .
    let scale = 0.3 / max;

    loop {
        previous_frame.cleanup_finished();

        if recreate_swapchain {
            vk_state.dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match vk_state
                .swapchain
                .recreate_with_dimension(vk_state.dimensions)
            {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err),
            };
            vk_state.swapchain = new_swapchain;

            let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                vk_state.device.clone(),
                &vs,
                &fs,
                &new_images,
                render_pass.clone(),
            );
            pipeline = new_pipeline;
            framebuffers = new_framebuffers;

            recreate_swapchain = false;
        }

        // !! important !! this is what gets fed to our friends
        // the vertex and frag shaders.
        let uniform_buffer_subbuffer = {
            let rotation =
                { Matrix3::from_angle_x(Rad(x_delta)) * Matrix3::from_angle_y(Rad(y_delta)) };
            // note: this teapot was meant for OpenGL where the origin is at the lower left
            //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
            let aspect_ratio = vk_state.dimensions[0] as f32 / vk_state.dimensions[1] as f32;

            let proj = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2 / 2.0),
                aspect_ratio,
                0.01,
                100.0,
            );

            let view = Matrix4::look_at(
                Point3::new(0.3, 0.3, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            );

            let scale = Matrix4::from_scale(scale);

            let uniform_data = vs::ty::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: (proj).into(),
                translate: translate.into(),
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        let set = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)
                .add_buffer(uniform_buffer_subbuffer)
                .unwrap()
                .build()
                .unwrap(),
        );

        // let tex_set = Arc::new(
        //     PersistentDescriptorSet::start(pipeline.clone(), 1)
        //         .add_sampled_image(texture.clone(), sampler.clone())
        //         .unwrap()
        //         .build()
        //         .unwrap(),
        // );

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(vk_state.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            vk_state.device.clone(),
            vk_state.queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            framebuffers[image_num].clone(),
            false,
            vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
        )
        .unwrap()
        .draw_indexed(
            pipeline.clone(),
            &DynamicState::none(),
            vec![vertex_buffer.clone(), normals_buffer.clone()],
            index_buffer.clone(),
            set.clone(),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();

        let future = previous_frame
            .join(acquire_future)
            .then_execute(vk_state.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                vk_state.queue.clone(),
                vk_state.swapchain.clone(),
                image_num,
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame = Box::new(future) as Box<_>;
            }
            Err(sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame = Box::new(sync::now(vk_state.device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame = Box::new(sync::now(vk_state.device.clone())) as Box<_>;
            }
        }

        let mut done = false;

        // the loop that parses user events
        // very simple and high level! I love it.
        vk_state.events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::CloseRequested,
                ..
            } => done = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Resized(_),
                ..
            } => recreate_swapchain = true,
            winit::Event::WindowEvent {
                event: winit::WindowEvent::MouseInput { state: s, .. },
                ..
            } => (mouse_state = s),
            winit::Event::DeviceEvent {
                event: winit::DeviceEvent::MouseMotion { delta: (x, y), .. },
                ..
            } => match mouse_state {
                winit::ElementState::Pressed => {
                    x_delta += DAMPENING * y as f32;
                    y_delta -= DAMPENING * x as f32;
                }
                winit::ElementState::Released => {}
            },
            _ => (),
        });
        if done {
            return;
        }
    }

}

/// A window resizing function , nithing to do with the loop.
/// I did not write this shit it is beyond me. It also
/// SEG FAULTS HARDCORE...
/// TODO: make this never get called.
fn window_size_dependent_setup(
    device: Arc<Device>,
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
) -> (
    Arc<GraphicsPipelineAbstract + Send + Sync>,
    Vec<Arc<FramebufferAbstract + Send + Sync>>,
) {
    let dimensions = images[0].dimensions();

    let depth_buffer =
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

    let framebuffers = images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>();

    // In the triangle example we use a dynamic viewport, as its a simple example.
    // However in the teapot example, we recreate the pipelines with a hardcoded viewport instead.
    // This allows the driver to optimize things, at the cost of slower window resizes.
    // https://computergraphics.stackexchange.com/questions/5742/vulkan-best-way-of-updating-pipeline-viewport
    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input(TwoBuffersDefinition::<geometry::Vertex, geometry::Normal>::new())
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }))
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    (pipeline, framebuffers)
}
