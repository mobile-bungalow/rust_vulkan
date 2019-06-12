/// Imports for loading files
use std::path::Path;

use std::iter;

/// types from the object parser
use tobj;

/// This is a placeholder, should be removed once the obj parser is up and running!
mod placeholder;

/// Vulkan imports, these are manifold , low level, and sinful.
use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;

use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

use std::sync::Arc;

/// arg parse
use clap::{App, Arg};

mod inserts;

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
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
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

    // break the positions up into groups of three
    let model_verts: Vec<placeholder::Vertex> = geom[0]
        .mesh
        .positions
        .chunks(3)
        .map(|chunk| placeholder::Vertex {
            position: (chunk[0], chunk[1], chunk[2]),
        })
        .collect();

    // init to zeroes
    let mut model_normals: Vec<placeholder::Normal> = vec![
        placeholder::Normal {
            normal: (0.0, 0.0, 0.0),
        };
        model_verts.len()
    ];

    geom[0]
        .mesh
        .indices
        .chunks(3)
        .map(|a| ((a[0]) as usize, a[1] as usize, a[2] as usize))
        .for_each(|(i_a, i_b, i_c)| {
            // the indices into the model vertex triples vector
            // which indicates one vector of a face

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

            let cross = (v_b - v_a).cross(v_c - v_b);

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
    for i in 0..model_normals.len() {
        let mut vec = cgmath::Vector3::new(
            model_normals[i].normal.0,
            model_normals[i].normal.1,
            model_normals[i].normal.2,
        );
        vec /= (vec.x.powf(2.0) + vec.y.powf(2.0) + vec.z.powf(2.0)).sqrt();
        model_normals[i].normal.0 = vec.x;
        model_normals[i].normal.1 = vec.y;
        model_normals[i].normal.2 = vec.z;
    }

    let extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, &extensions, None).unwrap();

    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    let mut events_loop = winit::EventsLoop::new();

    let surface = winit::WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();
    let window = surface.window();

    // unlike the triangle example we need to keep track of the width and height so we can calculate
    // render the teapot with the correct aspect ratio.
    let mut dimensions = if let Some(dimensions) = window.get_inner_size() {
        let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
        [dimensions.0, dimensions.1]
    } else {
        return;
    };

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .unwrap();

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let format = caps.supported_formats[0].0;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )
        .unwrap()
    };

    //let vertices = placeholder::VERTICES.iter().cloned();
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        model_verts.iter().cloned(),
    )
    .unwrap();

    //let normals = placeholder::NORMALS.iter().cloned();
    let normals_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        model_normals.iter().cloned(),
    )
    .unwrap();

    //let indices = placeholder::INDICES.iter().cloned();
    let index_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        geom[0].mesh.indices.iter().cloned(),
    )
    .unwrap();

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
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

    let (mut pipeline, mut framebuffers) =
        window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
    let mut recreate_swapchain = false;

    let mut previous_frame = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

    // these are used to rotate the world projection
    let mut y_delta: f32 = 0.0;
    let mut x_delta: f32 = 0.0;
    let mut mouse_state: winit::ElementState = winit::ElementState::Released;

    loop {
        previous_frame.cleanup_finished();

        if recreate_swapchain {
            dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err),
            };
            swapchain = new_swapchain;

            let (new_pipeline, new_framebuffers) = window_size_dependent_setup(
                device.clone(),
                &vs,
                &fs,
                &new_images,
                render_pass.clone(),
            );
            pipeline = new_pipeline;
            framebuffers = new_framebuffers;

            recreate_swapchain = false;
        }

        let uniform_buffer_subbuffer = {
            let rotation =
                { Matrix3::from_angle_x(Rad(x_delta)) * Matrix3::from_angle_y(Rad(y_delta)) };

            // note: this teapot was meant for OpenGL where the origin is at the lower left
            //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
            let aspect_ratio = dimensions[0] as f32 / dimensions[1] as f32;

            let proj =
                cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);

            let view = Matrix4::look_at(
                Point3::new(0.3, 0.3, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            );
            let scale = Matrix4::from_scale(0.1);

            let uniform_data = vs::ty::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: proj.into(),
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

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
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
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame = Box::new(future) as Box<_>;
            }
            Err(sync::FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame = Box::new(sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame = Box::new(sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
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
            .vertex_input(TwoBuffersDefinition::<
                placeholder::Vertex,
                placeholder::Normal,
            >::new())
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
