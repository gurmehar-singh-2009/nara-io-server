use glyphon::{
    Attrs, Cache, Color, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport,
};
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

pub struct RenderEntity<'a> {
    pub instance: EntityInstance,
    pub text: Option<&'a TextComponent>,
}

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

    // glyphon text rendering fields
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: glyphon::Buffer,
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
        ];

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(sample_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../../assets/Exo2.ttf").to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let mut atlas = TextAtlas::new(&device, &queue, &cache, srgb_format);
        let text_renderer = TextRenderer::new(
            &mut atlas,
            &device,
            MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );
        let mut viewport = Viewport::new(&device, &cache);
        viewport.update(
            &queue,
            Resolution {
                width: physical_width,
                height: physical_height,
            },
        );

        let mut text_buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(32.0, 38.0));
        text_buffer.set_size(Some(physical_width as f32), Some(physical_height as f32));
        text_buffer.set_text(
            "Hello WebGPU!",
            &Attrs::new().family(glyphon::Family::SansSerif),
            Shaping::Basic,
            None,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

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
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
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

            self.viewport.update(
                &self.queue,
                Resolution {
                    width: physical_width,
                    height: physical_height,
                },
            );

            self.text_buffer
                .set_size(Some(physical_width as f32), Some(physical_height as f32));

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

    pub fn render_entities(&mut self, instances: &[EntityInstance]) {
        self.window.request_redraw();

        if !self.is_surface_configured || instances.is_empty() {
            return;
        }

        self.update(instances);

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture)
            | CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Lost => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.view_formats[0]),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Entity Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Entity Render Pass"),
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

    pub fn render_entities_with_text(
        &mut self,
        entities: &[RenderEntity],
        camera_pos: [f32; 2],
        zoom: f32,
    ) {
        self.window.request_redraw();

        if !self.is_surface_configured || entities.is_empty() {
            return;
        }

        let instances: Vec<EntityInstance> = entities.iter().map(|e| e.instance).collect();
        self.update(&instances);

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture)
            | CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation
            | wgpu::CurrentSurfaceTexture::Lost => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.view_formats[0]),
            ..Default::default()
        });

        let screen_w = self.config.width as f32;
        let screen_h = self.config.height as f32;
        let aspect_ratio = if screen_h > 0.0 {
            screen_w / screen_h
        } else {
            1.0
        };

        let mut text_areas = Vec::new();

        for entity in entities {
            if let Some(text_comp) = entity.text {
                let (screen_x, screen_y) = Renderer::world_to_screen(
                    entity.instance.position,
                    camera_pos,
                    zoom,
                    aspect_ratio,
                    screen_w,
                    screen_h,
                );

                text_areas.push(TextArea {
                    buffer: &text_comp.buffer,
                    left: screen_x + text_comp.offset[0],
                    top: screen_y + text_comp.offset[1],
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: self.config.width as i32,
                        bottom: self.config.height as i32,
                    },
                    default_color: text_comp.color,
                    custom_glyphs: &[],
                });
            }
        }

        let _ = self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Entities + Text Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Entities + Text Render Pass"),
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

            self.text_renderer
                .render(&self.atlas, &self.viewport, &mut render_pass)
                .unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        self.atlas.trim();
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

    pub fn world_to_screen(
        world_pos: [f32; 2],
        camera_pos: [f32; 2],
        zoom: f32,
        aspect_ratio: f32,
        screen_width: f32,
        screen_height: f32,
    ) -> (f32, f32) {
        let ndc_x = ((world_pos[0] - camera_pos[0]) * zoom) / aspect_ratio;
        let ndc_y = (world_pos[1] - camera_pos[1]) * zoom;

        let screen_x = (ndc_x + 1.0) * (screen_width / 2.0);
        let screen_y = (1.0 - ndc_y) * (screen_height / 2.0);

        (screen_x, screen_y)
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
                let camera_pos = [0.0, 0.0];
                let zoom = 0.002;

                let dark_border = [0.22, 0.22, 0.22, 1.0];
                let grid_line_color = [0.75, 0.75, 0.75, 1.0];

                let mut player_label =
                    TextComponent::new(&mut state.font_system, "Player (Circle)");
                player_label.color = Color::rgb(30, 30, 30);
                player_label.set_centered_offset(-70.0);

                let mut enemy_label = TextComponent::new(&mut state.font_system, "Enemy (Circle)");
                enemy_label.color = Color::rgb(220, 40, 40);
                enemy_label.set_centered_offset(-70.0);

                let mut box_label = TextComponent::new(&mut state.font_system, "Box");
                box_label.color = Color::rgb(40, 180, 40);
                box_label.set_centered_offset(-70.0);

                let mut triangle_label =
                    TextComponent::new(&mut state.font_system, "Triangle (3-Polygon)");
                triangle_label.color = Color::rgb(220, 160, 20);
                triangle_label.set_centered_offset(-70.0);

                let mut pentagon_label =
                    TextComponent::new(&mut state.font_system, "Pentagon (5-Polygon)");
                pentagon_label.color = Color::rgb(160, 40, 220);
                pentagon_label.set_centered_offset(-70.0);

                let mut hexagon_label =
                    TextComponent::new(&mut state.font_system, "Hexagon (6-Polygon)");
                hexagon_label.color = Color::rgb(20, 180, 220);
                hexagon_label.set_centered_offset(-70.0);

                let mut square_label =
                    TextComponent::new(&mut state.font_system, "Square (4-Polygon)");
                square_label.color = Color::rgb(20, 180, 220);
                square_label.set_centered_offset(-70.0);

                let mut trapezoid_label = TextComponent::new(&mut state.font_system, "Trapezoid");
                trapezoid_label.color = Color::rgb(220, 100, 30);
                trapezoid_label.set_centered_offset(-70.0);

                let sample_entities = [
                    RenderEntity {
                        instance: EntityInstance {
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
                        text: None,
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [0.0, 35.0],
                            size: [30.0, 70.0],
                            rotation: 0.0,
                            shape_type: 1,
                            sides: 0,
                            fill_color: [0.55, 0.55, 0.55, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.0,
                            extra_param: 1.0,
                        },
                        text: None,
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [0.0, 0.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 0,
                            sides: 0,
                            fill_color: [0.2, 0.65, 0.95, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&player_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [300.0, 150.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 0,
                            sides: 0,
                            fill_color: [0.95, 0.25, 0.25, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&enemy_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [-300.0, 150.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 1,
                            sides: 0,
                            fill_color: [0.3, 0.85, 0.4, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&box_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [-300.0, -150.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 3,
                            sides: 3,
                            fill_color: [0.95, 0.7, 0.2, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&triangle_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [0.0, 200.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 3,
                            sides: 4,
                            fill_color: [0.95, 0.7, 0.2, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&square_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [0.0, -150.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 3,
                            sides: 5,
                            fill_color: [0.7, 0.3, 0.9, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&pentagon_label),
                    },
                    RenderEntity {
                        instance: EntityInstance {
                            position: [300.0, -150.0],
                            size: [80.0, 80.0],
                            rotation: 0.0,
                            shape_type: 3,
                            sides: 6,
                            fill_color: [0.2, 0.8, 0.9, 1.0],
                            border_color: dark_border,
                            border_thickness: 3.5,
                            extra_param: 1.0,
                        },
                        text: Some(&hexagon_label),
                    },
                ];

                state.render_entities_with_text(&sample_entities, camera_pos, zoom);
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

pub struct TextComponent {
    pub buffer: glyphon::Buffer,
    pub color: Color,
    pub offset: [f32; 2],
}

impl TextComponent {
    pub fn new(font_system: &mut FontSystem, initial_text: &str) -> Self {
        let mut buffer = glyphon::Buffer::new(font_system, Metrics::new(72.0, 80.0));

        buffer.set_text(
            initial_text,
            &Attrs::new().family(glyphon::Family::SansSerif),
            Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(font_system, false);

        let mut component = Self {
            buffer,
            color: Color::rgb(0, 0, 0),
            offset: [0.0, 0.0],
        };

        component.set_centered_offset(-60.0);
        component
    }

    pub fn measure(&self) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in self.buffer.layout_runs() {
            width = width.max(run.line_w);
            height += run.line_height;
        }

        (width, height)
    }

    pub fn set_centered_offset(&mut self, y_offset: f32) {
        let (width, _) = self.measure();
        self.offset = [-width / 2.0, y_offset];
    }

    pub fn update_text(&mut self, font_system: &mut FontSystem, new_text: &str) {
        self.buffer.set_text(
            new_text,
            &Attrs::new().family(glyphon::Family::SansSerif),
            Shaping::Basic,
            None,
        );

        self.buffer.shape_until_scroll(font_system, false);
    }
}
