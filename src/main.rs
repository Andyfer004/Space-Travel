use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod shaders;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    color: [f32; 4],
    time: f32, // Agregamos tiempo dinámico para animaciones
}

impl Uniforms {
    fn new(position: [f32; 3], color: [f32; 4], scale: f32) -> Self {
        let view = cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(0.0, 0.0, 28.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::unit_y(),
        );

        let aspect_ratio = 800.0 / 600.0;
        let fov = std::f32::consts::PI / 3.0;
        let proj = cgmath::perspective(cgmath::Deg(fov.to_degrees()), aspect_ratio, 0.1, 100.0);

        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            position[0],
            position[1],
            position[2],
        ));

        Self {
            view_proj: (proj * view).into(),
            model: (translation * cgmath::Matrix4::from_scale(scale)).into(),
            color,
            time: 0.0, // Inicializamos el tiempo en 0
        }
    }
}

struct Sphere {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniforms: Uniforms, // Guardamos los uniforms localmente
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    depth_view: wgpu::TextureView,
    spheres: Vec<Sphere>,
}

impl State {
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::VERTEX_SHADER.into()),
        });

        let shaders = vec![
            shaders::FRAGMENT_SHADER_SUN,
            shaders::FRAGMENT_SHADER_2,
            shaders::FRAGMENT_SHADER_3,
            shaders::FRAGMENT_SHADER_4,
            shaders::FRAGMENT_SHADER_5,
            shaders::FRAGMENT_SHADER_6,
            shaders::FRAGMENT_SHADER_7,
            shaders::FRAGMENT_SHADER_8,
        ];

        let positions = vec![
            [-15.0, 0.0, 0.0], // Sol
            [-8.5, 0.0, 0.0], // Mercurio
            [-6.0, 0.0, 0.0], // Venus
            [-3.0, 0.0, 0.0], // Tierra
            [0.0, 0.0, 0.0],  // Marte
            [3.0, 0.0, 0.0],  // Júpiter
            [7.0, 0.0, 0.0],  // Saturno
            [10.0, 0.0, 0.0], // Urano
        ];

        let colors = [
            [1.0, 0.9, 0.0, 1.0], // Sol
            [0.5, 0.5, 1.0, 1.0], // Mercurio
            [0.8, 0.5, 0.2, 1.0], // Venus
            [0.0, 0.5, 1.0, 1.0], // Tierra
            [1.0, 0.3, 0.3, 1.0], // Marte
            [0.3, 1.0, 0.3, 1.0], // Júpiter
            [0.5, 0.2, 0.7, 1.0], // Saturno
            [0.7, 0.7, 0.7, 1.0], // Urano
        ];

        let scales = [
            4.5, // Sol
            0.6, // Mercurio
            0.9, // Venus
            1.05, // Tierra
            0.75, // Marte
            1.5, // Júpiter
            1.2, // Saturno
            1.05, // Urano
        ];

        let mut spheres = Vec::new();

        for (i, position) in positions.iter().enumerate() {
            if i >= shaders.len() {
                break;
            }

            let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Fragment Shader"),
                source: wgpu::ShaderSource::Wgsl(shaders[i].into()),
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("Pipeline {}", i)),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                })),                
                vertex: wgpu::VertexState {
                    module: &vertex_shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

            let uniforms = Uniforms::new(*position, colors[i], scales[i]);

            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            let (vertices, indices) = generate_sphere(60, 60);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            spheres.push(Sphere {
                pipeline,
                uniform_buffer,
                bind_group,
                vertex_buffer,
                index_buffer,
                num_indices: indices.len() as u32,
                uniforms,
            });
        }

        Self {
            device,
            queue,
            surface,
            config,
            depth_view,
            spheres,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        for sphere in &self.spheres {
            render_pass.set_pipeline(&sphere.pipeline);
            render_pass.set_bind_group(0, &sphere.bind_group, &[]);
            render_pass.set_vertex_buffer(0, sphere.vertex_buffer.slice(..));
            render_pass.set_index_buffer(sphere.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..sphere.num_indices, 0, 0..1);
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn generate_sphere(stacks: usize, slices: usize) -> (Vec<[f32; 3]>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI * i as f32 / stacks as f32 - std::f32::consts::PI / 2.0;
        let xy = stack_angle.cos();
        let z = stack_angle.sin();

        for j in 0..=slices {
            let slice_angle = 2.0 * std::f32::consts::PI * j as f32 / slices as f32;
            let x = xy * slice_angle.cos();
            let y = xy * slice_angle.sin();
            vertices.push([x, y, z]);
        }
    }

    for i in 0..stacks {
        for j in 0..slices {
            let first = i * (slices + 1) + j;
            let second = first + slices + 1;

            indices.push(first as u16);
            indices.push(second as u16);
            indices.push(first as u16 + 1);

            indices.push(second as u16);
            indices.push(second as u16 + 1);
            indices.push(first as u16 + 1);
        }
    }

    (vertices, indices)
}



fn main() {
    pollster::block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    let mut current_time: f32 = 0.0;
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                current_time += 0.016; // Incrementa ~60 FPS
                for sphere in &mut state.spheres {
                    // Escribe el tiempo en el buffer de uniformes (color.a)
                    let mut uniforms = sphere.uniforms;
                    uniforms.color[3] = current_time; // Usa color.a para pasar el tiempo
                    state.queue.write_buffer(
                        &sphere.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[uniforms]),
                    );
                }
                window.request_redraw();
            }            
            Event::RedrawRequested(_) => {
                state.render().unwrap();
            }
            _ => {}
        }
    });
}
