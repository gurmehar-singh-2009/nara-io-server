use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wgpu::util::DeviceExt;
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    FragmentState, Instance, InstanceDescriptor, Limits, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderStages, Surface,
    SurfaceConfiguration, VertexState,
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    platform::web::*,
    window::Window,
};

use crate::render::buffers::{CameraUniform, EntityInstance};

pub struct RenderState {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    num_instances: u32,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let scale_factor = window.scale_factor();
        let inner_size = window.inner_size();

        let physical_width = (inner_size.width as f64 * scale_factor) as u32;
        let physical_height = (inner_size.height as f64 * scale_factor) as u32;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let srgb_format = surface_format.add_srgb_suffix();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_width,
            height: physical_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![srgb_format],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let aspect_ratio = if size.height > 0 {
            size.width as f32 / size.height as f32
        } else {
            1.0
        };
        let camera_uniform = CameraUniform {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            camera_pos: [0.0, 0.0],
            zoom: 0.002,
            aspect_ratio,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let vs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader/vertex.wgsl").into()),
        });

        let fs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader/fragment.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &vs_module,
                entry_point: Some("vs_main"),
                buffers: &[Some(EntityInstance::desc())],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &fs_module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: srgb_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
            primitive: PrimitiveState::default(),
        });

        let dark_border = [0.22, 0.22, 0.22, 1.0];
        let grid_line_color = [0.75, 0.75, 0.75, 1.0];

        let sample_instances: &[EntityInstance] = &[
            EntityInstance {
                position: [0.0, 0.0],
                size: [10000.0, 10000.0],
                rotation: 0.0,
                shape_type: 2,
                sides: 0,
                fill_color: [0.808, 0.808, 0.808, 1.0],
                border_color: grid_line_color,
                border_thickness: 2.0,
                extra_param: 64.0,
            },
            EntityInstance {
                position: [0.0, 40.0],
                size: [36.0, 60.0],
                rotation: 0.0,
                shape_type: 1,
                sides: 0,
                fill_color: [0.6, 0.6, 0.6, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [0.0, 0.0],
                size: [75.0, 75.0],
                rotation: 0.0,
                shape_type: 0,
                sides: 0,
                fill_color: [0.0, 0.7, 1.0, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [240.0, 120.0],
                size: [32.0, 50.0],
                rotation: 0.785,
                shape_type: 1,
                sides: 0,
                fill_color: [0.6, 0.6, 0.6, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [270.0, 150.0],
                size: [85.0, 85.0],
                rotation: 0.0,
                shape_type: 0,
                sides: 0,
                fill_color: [0.95, 0.3, 0.3, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [-180.0, -140.0],
                size: [50.0, 50.0],
                rotation: 0.35,
                shape_type: 1,
                sides: 0,
                fill_color: [0.98, 0.88, 0.35, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [-220.0, 160.0],
                size: [55.0, 55.0],
                rotation: -0.2,
                shape_type: 1,
                sides: 0,
                fill_color: [0.98, 0.88, 0.35, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [160.0, -180.0],
                size: [65.0, 65.0],
                rotation: 0.6,
                shape_type: 3,
                sides: 3,
                fill_color: [0.95, 0.45, 0.65, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [-320.0, -60.0],
                size: [105.0, 105.0],
                rotation: 0.0,
                shape_type: 3,
                sides: 7,
                fill_color: [0.45, 0.35, 0.95, 1.0],
                border_color: dark_border,
                border_thickness: 3.0,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [70.0, 110.0],
                size: [22.0, 22.0],
                rotation: 0.0,
                shape_type: 0,
                sides: 0,
                fill_color: [0.0, 0.7, 1.0, 1.0],
                border_color: dark_border,
                border_thickness: 2.5,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [120.0, 170.0],
                size: [22.0, 22.0],
                rotation: 0.0,
                shape_type: 0,
                sides: 0,
                fill_color: [0.0, 0.7, 1.0, 1.0],
                border_color: dark_border,
                border_thickness: 2.5,
                extra_param: 1.0,
            },
            EntityInstance {
                position: [170.0, 230.0],
                size: [22.0, 22.0],
                rotation: 0.0,
                shape_type: 0,
                sides: 0,
                fill_color: [0.0, 0.7, 1.0, 1.0],
                border_color: dark_border,
                border_thickness: 2.5,
                extra_param: 1.0,
            },
        ];

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(sample_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            render_pipeline,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            num_instances: sample_instances.len() as u32,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let scale_factor = self.window.scale_factor();

        let physical_width = (width as f64 * scale_factor) as u32;
        let physical_height = (height as f64 * scale_factor) as u32;

        if physical_width > 0 && physical_height > 0 {
            self.config.width = physical_width;
            self.config.height = physical_height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            let aspect_ratio = physical_width as f32 / physical_height as f32;

            let camera_uniform = CameraUniform {
                view_proj: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                camera_pos: [0.0, 0.0],
                zoom: 0.002,
                aspect_ratio,
            };

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );
        }
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    pub fn render(&mut self) {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return;
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => return,
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.view_formats[0]),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.808,
                            g: 0.808,
                            b: 0.808,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

            render_pass.draw(0..6, 0..self.num_instances);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);
    }

    pub fn update(&mut self, instances: &[EntityInstance]) {
        self.num_instances = instances.len() as u32;

        if instances.is_empty() {
            return;
        }

        let raw_data = bytemuck::cast_slice(instances);
        let required_size = raw_data.len() as u64;

        if required_size > self.instance_buffer.size() {
            self.instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Dynamic Instance Buffer"),
                        contents: raw_data,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
        } else {
            self.queue.write_buffer(&self.instance_buffer, 0, raw_data);
        }
    }
}

pub struct Renderer {
    proxy: Option<EventLoopProxy<RenderState>>,
    state: Option<RenderState>,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<RenderState>) -> Self {
        Self {
            proxy: Some(event_loop.create_proxy()),
            state: None,
        }
    }
}

impl ApplicationHandler<RenderState> for Renderer {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = wgpu::web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas: web_sys::HtmlCanvasElement = document
            .get_element_by_id("gameCanvas")
            .unwrap_throw()
            .unchecked_into();

        let dpr = window.device_pixel_ratio();

        let client_width = canvas.client_width() as f64;
        let client_height = canvas.client_height() as f64;

        canvas.set_width((client_width * dpr) as u32);
        canvas.set_height((client_height * dpr) as u32);

        let mut window_attribs = Window::default_attributes();

        let window = wgpu::web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas = document.get_element_by_id("gameCanvas").unwrap_throw();
        let html_canvas_element = canvas.unchecked_into();
        window_attribs = window_attribs.with_canvas(Some(html_canvas_element));

        let window = Arc::new(event_loop.create_window(window_attribs).unwrap());

        if let Some(proxy) = self.proxy.take() {
            spawn_local(async move {
                assert!(proxy.send_event(RenderState::new(window).await).is_ok())
            });
        }
    }

    fn user_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        mut event: RenderState,
    ) {
        event.window.request_redraw();

        event.resize(
            event.window.inner_size().width,
            event.window.inner_size().height,
        );
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(c) => c,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}
