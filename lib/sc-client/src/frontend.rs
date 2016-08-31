use std::sync::Arc;
use std::time::Duration;
use cgmath::{self, Matrix4, Vector2, Vector3, Rad, SquareMatrix};
use winit::{self, Event, ElementState, VirtualKeyCode};
use vulkano;
use vulkano::buffer::cpu_access::CpuAccessibleBuffer;
use vulkano::command_buffer::{self, DynamicState, PrimaryCommandBufferBuilder};
use vulkano::device::Device;
use vulkano::format::D16Unorm;
use vulkano::framebuffer::{Framebuffer, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineParams};
use vulkano::pipeline::vertex::TwoBuffersDefinition;
use vulkano::pipeline::raster::{Rasterization, CullMode, FrontFace};
use vulkano::swapchain::{Swapchain, PresentMode};
use vulkano_win::{self, VkSurfaceBuild};

use {vs, fs, teapot};
use sc_client_game::ClientWorld;
use sc_input_data::Button;

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

pub struct Frontend {
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
    swapchain: Arc<Swapchain>,
    submissions: Vec<Arc<vulkano::command_buffer::Submission>>,
}

impl Frontend {
    pub fn init() -> Self {
        let extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &extensions, None)
            .expect("failed to create instance");

        let physical = PhysicalDevice::enumerate(&instance)
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
            let present = PresentMode::Fifo;
            let usage = caps.supported_usage_flags;
            let format = caps.supported_formats[0].0;

            Swapchain::new(
                &device, &window.surface(), 3, format, dimensions, 1,
                &usage, &queue, vulkano::swapchain::SurfaceTransform::Identity,
                vulkano::swapchain::CompositeAlpha::Opaque,
                present, true, None
            ).expect("failed to create swapchain")
        };

        let depth_buffer = AttachmentImage::transient(
            &device, images[0].dimensions(), D16Unorm
        ).unwrap();

        let vertex_buffer = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                &device,
                teapot::VERTICES.len(),
                &vulkano::buffer::BufferUsage::all(),
                Some(queue.family())
            ).expect("failed to create buffer")
        };

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

        let pipeline = GraphicsPipeline::new(&device, GraphicsPipelineParams {
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
            raster: Rasterization {
                cull_mode: CullMode::Back,
                front_face: FrontFace::Clockwise, // This seems to cull CCW, I'm not sure why
                .. Default::default()
            },
            multisample: vulkano::pipeline::multisample::Multisample::disabled(),
            fragment_shader: fs.main_entry_point(),
            depth_stencil: vulkano::pipeline::depth_stencil::DepthStencil::simple_depth_test(),
            blend: vulkano::pipeline::blend::Blend::pass_through(),
            layout: &pipeline_layout,
            render_pass: Subpass::from(&renderpass, 0).unwrap(),
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

        Frontend {
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

    pub fn poll_events<H: FnMut(FrontendEvent)>(&self, mut handler: H) {
        // Handle the window's events
        for ev in self.window.window().poll_events() {
            match ev {
                Event::Closed => handler(FrontendEvent::Close),
                Event::KeyboardInput(state, _, Some(key)) => {
                    // Translate the keyboard event to a button event
                    let down = state == ElementState::Pressed;
                    let button = match key {
                        VirtualKeyCode::W => Some(Button::MoveForward),
                        VirtualKeyCode::S => Some(Button::MoveBackward),
                        VirtualKeyCode::D => Some(Button::MoveRight),
                        VirtualKeyCode::A => Some(Button::MoveLeft),
                        VirtualKeyCode::Escape => Some(Button::Menu),
                        _ => None
                    };

                    // If we were able to translate it, send it over
                    if let Some(button) = button {
                        handler(FrontendEvent::ButtonState(button, down));
                    }
                },
                Event::MouseMoved(position) =>
                    handler(FrontendEvent::MouseMove(position.into())),
                _ => ()
            }
        }
    }

    pub fn render(&mut self, world: &ClientWorld) {
        // Remove all command buffers that the GPU is finished with
        self.submissions.retain(|s| s.destroying_would_block());

        // Aquire ownership of the next frame's image to work on
        let image_num = self.swapchain.acquire_next_image(Duration::from_millis(1))
            .expect("Unable to aquire swapchain image in time.");

        // Calculate the camera projection matrix
        let mut proj = cgmath::perspective(
            Rad(3.141592 / 2.0),
            { let d = &self.dimensions; d[0] as f32 / d[1] as f32 },
            0.01, 100.0
        );

        // Correct the projection for the inverted Y in vulkan
        proj = Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0) * proj;

        // Calculate the camera view matrix
        let cam = world.camera();
        let translation = Matrix4::from_translation(cam.position());
        let rotation_yaw = Matrix4::from_angle_y(Rad(cam.yaw()));
        let view = (translation * rotation_yaw).invert().unwrap();

        // Calculate the teapot model matrix
        let translation = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0));
        let rotation = Matrix4::from_angle_y(Rad(world.teapot()));
        let scale = Matrix4::from_scale(0.01);
        let model = translation * rotation * scale;

        let uniform_buffer = unsafe {
            CpuAccessibleBuffer::<vs::ty::Data>::uninitialized(
                &self.device,
                &vulkano::buffer::BufferUsage::all(),
                Some(self.queue.family())
            ).expect("failed to create buffer")
        };

        {
            let mut mapping = uniform_buffer.write(Duration::new(0, 0)).unwrap();
            mapping.worldview = (view * model).into();
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
                &DynamicState::none(), &set, &()
            )
            .draw_end()
            .build();

        // Submit the command buffer and keep track of the submission so we can clean it up later
        let submission = command_buffer::submit(&buffer, &self.queue).unwrap();
        self.submissions.push(submission);

        // Present our new frame to the user
        self.swapchain.present(&self.queue, image_num).unwrap();
    }
}

pub enum FrontendEvent {
    Close,
    ButtonState(Button, bool),
    MouseMove(Vector2<i32>),
}
