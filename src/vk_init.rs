#![allow(unused_imports)]

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use vulkano_win::VkSurfaceBuild;

use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

use std::sync::Arc;

/// struct which contains all of the important parts of the vulkan set up
pub struct VKState<'a> {
    vk: Arc<Instance>, //? The root instance of vulkan, used for a lot of other calls
    device: PhysicalDevice<'a>,
}

impl VKState<'static> {

        // initialize gpu (device)
        let dev = PhysicalDevice::enumerate(&vk).next().unwrap();

        let vkstate = VKState {
            vk: vk.clone(),
            device: dev,
        };
        vkstate
    }
    pub fn win_setup(vk: &mut VKState) {
        let mut events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, vk.vk.clone())
            .unwrap();
        let window = surface.window();
    }
}
