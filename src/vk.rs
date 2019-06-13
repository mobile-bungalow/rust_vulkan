use std::sync::Arc;
/// Vulkan imports, these are manifold , low level, and sinful.
use vulkano::device::{Device, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{PresentMode, SurfaceTransform, Swapchain};
use vulkano_win::VkSurfaceBuild;

#[derive(Debug)]
pub enum VKError {
    VKWindowError,
    //VKDeviceError,
}

/// main struct that holds the initiliazed vulkan values.
pub struct VKState {
    /// TODO: find out what this is... I think it is the literal next frame?
    pub images: Vec<std::sync::Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    /// communication with the winit frame buffer;
    pub swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
    /// async reference to the hardware we are running on
    pub device: Arc<vulkano::device::Device>,
    /// async reference the the command queue, we buffer commands in to this then run them async with a fence
    pub queue: Arc<vulkano::device::Queue>,
    /// main winit event loop. controls things like user input
    pub events_loop: winit::EventsLoop,
    /// TODO : figure out what this actually does
    pub surface: Arc<vulkano::swapchain::Surface<winit::Window>>,
    // window dimensions
    pub dimensions: [u32; 2],
}

impl VKState {
    /// set up the vulkan environment. may panic in a misconfigured environment,
    /// make sure you have the vulkan library somewhere in your path.
    pub fn vk_init() -> Result<Self, VKError> {
        let extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &extensions, None).unwrap();

        let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        let events_loop = winit::EventsLoop::new();
        let surface = winit::WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        let window = surface.window();

        // unlike the triangle example we need to keep track of the width and height so we can calculate
        // render the teapot with the correct aspect ratio.
        let dimensions = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        } else {
            return Err(VKError::VKWindowError);
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
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let format = caps.supported_formats[0].0;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        let queue = queues.next().unwrap();

        let (swapchain, images) = {
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

        Ok(VKState {
            images,
            swapchain,
            device,
            queue,
            surface,
            events_loop,
            dimensions,
        })
    }
}
