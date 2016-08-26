use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;
use cgmath;
use winit::{self, Event};
use vulkano;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::PrimaryCommandBufferBuilder;
use vulkano::device::Device;
use vulkano::framebuffer::Framebuffer;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano_win::{self, VkSurfaceBuild};

use {FrontendEvent, FrontendCommand};
use framecounter::FrameCounter;
use teapot;
use {vs, fs};

pub fn frontend_runtime(sender: Sender<Vec<FrontendEvent>>, receiver: Receiver<FrontendCommand>) {
    let mut runtime = FrontendRuntime::init();
    let mut counter = FrameCounter::new();

    let mut teapot = 0.0;

    loop {
        // Get the frontend events that have happened and send them over
        let events = runtime.poll_events();
        if events.len() != 0 {
            sender.send(events).unwrap();
        }

        // Check what the backend wants us to do
        if let Ok(command) = receiver.try_recv() {
            match command {
                FrontendCommand::Stop => return
            }
        }

        teapot += 0.05;

        runtime.render(teapot);

        counter.tick();
    }
}

mod renderpass {
    single_pass_renderpass!{
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: ::vulkano::format::Format,
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: ::vulkano::format::D16Unorm,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    }
}

mod pipeline_layout {
    pipeline_layout!{
        set0: {
            uniforms: UniformBuffer<::vs::ty::Data>
        }
    }
}

pub struct FrontendRuntime {
    window: vulkano_win::Window,
    dimensions: [u32; 2],

    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[teapot::Vertex]>>,
    normals_buffer: Arc<CpuAccessibleBuffer<[teapot::Normal]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,

    pipeline: Arc<GraphicsPipeline<TwoBuffersDefinition<teapot::Vertex, teapot::Normal>, pipeline_layout::CustomPipeline, renderpass::CustomRenderPass>>,
    pipeline_layout: Arc<pipeline_layout::CustomPipeline>,
    descriptor_pool: Arc<vulkano::descriptor::descriptor_set::DescriptorPool>,

    framebuffers: Vec<Arc<Framebuffer<renderpass::CustomRenderPass>>>,
    renderpass: Arc<renderpass::CustomRenderPass>,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
    submissions: Vec<Arc<vulkano::command_buffer::Submission>>,
}

impl FrontendRuntime {
    pub fn init() -> Self {
        let extensions = vulkano_win::required_extensions();
        let instance = vulkano::instance::Instance::new(None, &extensions, None)
            .expect("failed to create instance");

        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next()
            .expect("no device available");
        println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

        let window = winit::WindowBuilder::new().build_vk_surface(&instance).unwrap();

        let queue_families = physical.queue_families()
            .find(|q| q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false))
            .expect("couldn't find a graphical queue family");

        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let (device, mut queues) = Device::new(
            &physical, physical.supported_features(),
            &device_ext, [(queue_families, 0.5)].iter().cloned()
        ).expect("failed to create device");
        let queue = queues.next().unwrap();

        let (swapchain, images) = {
            let caps = window.surface().get_capabilities(&physical).expect("failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let present = vulkano::swapchain::PresentMode::Fifo;
            let usage = caps.supported_usage_flags;
            let format = caps.supported_formats[0].0;

            vulkano::swapchain::Swapchain::new(
                &device, &window.surface(), 3, format, dimensions, 1,
                &usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                vulkano::swapchain::CompositeAlpha::Opaque,
                present, true, None
            ).expect("failed to create swapchain")
        };



        let depth_buffer = vulkano::image::attachment::AttachmentImage::transient(&device, images[0].dimensions(), vulkano::format::D16Unorm).unwrap();

        let vertex_buffer = unsafe { CpuAccessibleBuffer
                                   ::uninitialized_array(&device, teapot::VERTICES.len(),
                                           &vulkano::buffer::BufferUsage::all(), Some(queue.family()))
                                           .expect("failed to create buffer") };

        {
            let mut mapping = vertex_buffer.write(Duration::new(0, 0)).unwrap();
            for (o, i) in mapping.iter_mut().zip(teapot::VERTICES.iter()) {
                *o = *i;
            }
        }

        let normals_buffer = unsafe { CpuAccessibleBuffer
                                    ::uninitialized_array(&device, teapot::NORMALS.len(),
                                            &vulkano::buffer::BufferUsage::all(), Some(queue.family()))
                                            .expect("failed to create buffer") };

        {
            let mut mapping = normals_buffer.write(Duration::new(0, 0)).unwrap();
            for (o, i) in mapping.iter_mut().zip(teapot::NORMALS.iter()) {
                *o = *i;
            }
        }

        let index_buffer = unsafe { CpuAccessibleBuffer
                                  ::uninitialized_array(&device, teapot::INDICES.len(),
                                          &vulkano::buffer::BufferUsage::all(), Some(queue.family()))
                                          .expect("failed to create buffer") };

        {
            let mut mapping = index_buffer.write(Duration::new(0, 0)).unwrap();
            for (o, i) in mapping.iter_mut().zip(teapot::INDICES.iter()) {
                *o = *i;
            }
        }

        let vs = vs::Shader::load(&device).expect("failed to create shader module");
        let fs = fs::Shader::load(&device).expect("failed to create shader module");

        let renderpass = renderpass::CustomRenderPass::new(&device, &renderpass::Formats {
            color: (images[0].format(), 1),
            depth: (vulkano::format::D16Unorm, 1)
        }).unwrap();

        let descriptor_pool = vulkano::descriptor::descriptor_set::DescriptorPool::new(&device);

        let pipeline_layout = pipeline_layout::CustomPipeline::new(&device).unwrap();

        let pipeline = vulkano::pipeline::GraphicsPipeline::new(&device, vulkano::pipeline::GraphicsPipelineParams {
            vertex_input: vulkano::pipeline::vertex::TwoBuffersDefinition::new(),
            vertex_shader: vs.main_entry_point(),
            input_assembly: vulkano::pipeline::input_assembly::InputAssembly::triangle_list(),
            tessellation: None,
            geometry_shader: None,
            viewport: vulkano::pipeline::viewport::ViewportsState::Fixed {
                data: vec![(
                    vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        depth_range: 0.0 .. 1.0,
                        dimensions: [images[0].dimensions()[0] as f32, images[0].dimensions()[1] as f32],
                    },
                    vulkano::pipeline::viewport::Scissor::irrelevant()
                )],
            },
            raster: Default::default(),
            multisample: vulkano::pipeline::multisample::Multisample::disabled(),
            fragment_shader: fs.main_entry_point(),
            depth_stencil: vulkano::pipeline::depth_stencil::DepthStencil::simple_depth_test(),
            blend: vulkano::pipeline::blend::Blend::pass_through(),
            layout: &pipeline_layout,
            render_pass: vulkano::framebuffer::Subpass::from(&renderpass, 0).unwrap(),
        }).unwrap();

        let framebuffers: Vec<_> = images.iter().map(|image| {
            let attachments = renderpass::AList {
                color: &image,
                depth: &depth_buffer,
            };

            Framebuffer::new(
                &renderpass,
                [image.dimensions()[0], image.dimensions()[1], 1],
                attachments
            ).unwrap()
        }).collect();

        FrontendRuntime {
            window: window,
            dimensions: images[0].dimensions(),

            device: device,
            queue: queue,

            vertex_buffer: vertex_buffer,
            normals_buffer: normals_buffer,
            index_buffer: index_buffer,

            pipeline: pipeline,
            pipeline_layout: pipeline_layout,
            descriptor_pool: descriptor_pool,

            framebuffers: framebuffers,
            renderpass: renderpass,
            swapchain: swapchain,
            submissions: Vec::new(),
        }
    }

    pub fn poll_events(&self) -> Vec<FrontendEvent> {
        let mut events = Vec::new();

        // Handle the window's events
        for ev in self.window.window().poll_events() {
            match ev {
                Event::Closed => events.push(FrontendEvent::Closed),
                _ => ()
            }
        }

        events
    }

    pub fn render(&mut self, teapot: f32) {
        // Remove all command buffers that the GPU is finished with
        self.submissions.retain(|s| s.destroying_would_block());

        // Aquire ownership of the next frame's image to work on
        let image_num = self.swapchain.acquire_next_image(Duration::from_millis(1))
            .expect("Unable to aquire swapchain image in time.");

        // Build up the uniforms for this frame
        // note: this teapot was meant for OpenGL where the origin is at the lower left
        //       instead the origin is at the upper left in vulkan, so we reverse the Y axis
        let proj = cgmath::perspective(
            cgmath::Rad(3.141592 / 2.0),
            { let d = &self.dimensions; d[0] as f32 / d[1] as f32 },
            0.01, 100.0
        );
        let view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(0.3, 0.3, 1.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -1.0, 0.0)
        );
        let scale = cgmath::Matrix4::from_scale(0.01);
        let rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(teapot));

        let uniform_buffer = unsafe {
            CpuAccessibleBuffer::<vs::ty::Data>::uninitialized(
                &self.device,
                &vulkano::buffer::BufferUsage::all(),
                Some(self.queue.family())
            ).expect("failed to create buffer")
        };

        {
            let mut mapping = uniform_buffer.write(Duration::new(0, 0)).unwrap();
            mapping.worldview = (view * scale * rotation).into();
            mapping.proj = proj.into();
        }

        let set = pipeline_layout::set0::Set::new(
            &self.descriptor_pool, &self.pipeline_layout,
            &pipeline_layout::set0::Descriptors {
                uniforms: &uniform_buffer
            }
        );

        // Build up the command buffer we want to submit for this frame
        let buffer = PrimaryCommandBufferBuilder::new(&self.device, self.queue.family())
            .draw_inline(&self.renderpass, &self.framebuffers[image_num], renderpass::ClearValues {
                 color: [0.1, 0.1, 0.1, 1.0],
                 depth: 1.0,
            })
            .draw_indexed(
                &self.pipeline, (&self.vertex_buffer, &self.normals_buffer), &self.index_buffer,
                &vulkano::command_buffer::DynamicState::none(), &set, &()
            )
            .draw_end()
            .build();

        // Submit the command buffer and keep track of the submission so we can clean it up later
        let submission = vulkano::command_buffer::submit(&buffer, &self.queue).unwrap();
        self.submissions.push(submission);

        // Present our new frame to the user
        self.swapchain.present(&self.queue, image_num).unwrap();
    }
}
