use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use rand::Rng;
mod shaders;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    color: [f32; 4],
    time: f32, // Agregamos tiempo dinámico para animaciones
    orbital_radius: f32,  // Añadimos el radio orbital
    orbital_speed: f32,   // Añadimos la velocidad orbital
}



// Primero agregamos una nueva estructura para manejar las estrellas
const STAR_COUNT: usize = 2000000;

struct Star {
    position: [f32; 3],
    brightness: f32,
}


fn load_obj_model(file_path: &str) -> (Vec<[f32; 3]>, Vec<u32>) {
    use tobj;
    use std::path::Path;

    // Cargar el archivo .obj
    let (models, _) = tobj::load_obj(Path::new(file_path), &tobj::LoadOptions::default())
        .expect("Failed to load OBJ file");

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Iterar sobre los modelos dentro del archivo OBJ
    for model in models {
        let mesh = &model.mesh;

        // Extraer vértices (posición)
        vertices.extend(
            mesh.positions
                .chunks(3) // Cada vértice tiene 3 componentes (x, y, z)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        );

        // Extraer índices
        indices.extend(mesh.indices.iter().map(|&index| index as u32));
    }

    (vertices, indices)
}


impl Star {
    fn new(rng: &mut impl rand::Rng) -> Self {
        let theta = rng.gen_range(0.0..std::f32::consts::PI * 2.0); // Ángulo azimutal (0 a 360°).
        let phi = rng.gen_range(-std::f32::consts::PI / 2.0..std::f32::consts::PI / 2.0); // Ángulo polar (-90° a 90°).
        let radius = rng.gen_range(5.0..20.0); // Radio de la esfera.

        Self {
            position: [
                radius * phi.cos() * theta.cos(), // Coordenada X.
                radius * phi.cos() * theta.sin(), // Coordenada Y.
                radius * phi.sin(),               // Coordenada Z.
            ],
            brightness: rng.gen_range(0.5..1.0),
        }
    }
}

struct Spaceship {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniforms: Uniforms,
}

impl Spaceship {
    fn new_from_obj(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        file_path: &str,
        scale: f32,
        color: [f32; 4],
    ) -> Self {
        // Load OBJ model
        let (vertices, indices) = load_obj_model(file_path);

        // Create uniform bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Spaceship Uniform Bind Group Layout"),
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
            label: Some("Spaceship Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::VERTEX_SHADER.into()),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Spaceship Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::FRAGMENT_SHADER_8.into()),
        });

        // Create pipeline layout with uniform bind group layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Spaceship Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Spaceship Pipeline"),
            layout: Some(&pipeline_layout),
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
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let uniforms = Uniforms::new([0.0, 0.0, 0.0], color, scale, 0.0, 0.0);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spaceship Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Spaceship Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spaceship Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spaceship Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Spaceship {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniforms,
        }
    }
}



// Shaders de las estrellas
// Actualización de los shaders de estrellas.
const STAR_VERTEX_SHADER: &str = r#"
@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let view_proj: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, -10.0, 1.0), // Ajusta la distancia de la cámara.
    );
    return view_proj * vec4<f32>(position, 1.0);
}
"#;

const STAR_FRAGMENT_SHADER: &str = r#"
@fragment
fn main() -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0); // Color blanco brillante para las estrellas.
}
"#;


impl Uniforms {
    fn new(position: [f32; 3], color: [f32; 4], scale: f32, orbital_radius: f32, orbital_speed: f32) -> Self {
        let view = cgmath::Matrix4::look_at_rh(
            cgmath::Point3::new(0.0, 5.0, 28.0),  // Ajustamos la cámara para mejor vista
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::unit_y(),
        );

        let aspect_ratio = 800.0 / 600.0;
        let proj = cgmath::perspective(cgmath::Deg(60.0), aspect_ratio, 0.1, 200.0); // Far plane mayor que 50.0


        Self {
            view_proj: (proj * view).into(),
            model: cgmath::Matrix4::from_scale(scale).into(),
            color,
            time: 0.0,
            orbital_radius,
            orbital_speed,
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
    star_buffer: wgpu::Buffer,
    num_stars: u32,
    star_pipeline: wgpu::RenderPipeline,
    spaceship: Spaceship, // Agrega este campo
    spaceship_position: cgmath::Vector3<f32>, // Posición de la nave
    spaceship_rotation: cgmath::Vector3<f32>,
}


impl State {

    fn render_stars(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Star Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Mantener contenido previo.
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
    
        render_pass.set_pipeline(&self.star_pipeline);
        render_pass.set_vertex_buffer(0, self.star_buffer.slice(..));
        render_pass.draw(0..self.num_stars, 0..1); // Asegúrate de que `num_stars` sea correcto.
    }
    

    fn generate_star_pipeline(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::RenderPipeline {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Star Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(STAR_VERTEX_SHADER.into()),
        });
    
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Star Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(STAR_FRAGMENT_SHADER.into()),
        });
    
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Star Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main", // Asegúrate de que coincida con el shader
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main", // Asegúrate de que coincida con el shader
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList, // Renderizar como puntos
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },            
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
    



    fn generate_stars(device: &wgpu::Device) -> (wgpu::Buffer, u32) {
        let mut rng = rand::thread_rng();
        let stars: Vec<[f32; 3]> = (0..STAR_COUNT)
            .map(|_| Star::new(&mut rng).position)
            .collect();
    
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Star Buffer"),
            contents: bytemuck::cast_slice(&stars),
            usage: wgpu::BufferUsages::VERTEX,
        });
    
        (buffer, STAR_COUNT as u32)
    }
    
    async fn new(window: &winit::window::Window) -> Self {
        
        
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
    
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device Descriptor"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            }, None)
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
    
        // Crear textura de profundidad
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        // Layout de uniformes
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
    
        // Shaders de esferas
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
            [-8.5, 0.0, 0.0],  // Mercurio
            [-6.0, 0.0, 0.0],  // Venus
            [-3.0, 0.0, 0.0],  // Tierra
            [0.0, 0.0, 0.0],   // Marte
            [3.0, 0.0, 0.0],   // Júpiter
            [7.0, 0.0, 0.0],   // Saturno
            [10.0, 0.0, 0.0],  // Urano
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
    
        let orbital_speeds = [
            0.0,  // Sol
            1.6,  // Mercurio
            1.2,  // Venus
            1.0,  // Tierra
            0.8,  // Marte
            0.4,  // Júpiter
            0.3,  // Saturno
            0.2,  // Urano
        ];
    
        let orbital_radii = [
            0.0,  // Sol
            7.0,  // Mercurio
            9.0,  // Venus
            11.0, // Tierra
            13.0, // Marte
            15.0, // Júpiter
            17.0, // Saturno
            19.0, // Urano
        ];
        let spaceship = Spaceship::new_from_obj(
            &device,
            &config,
            "assets/model3d.obj",
            0.5,                     // Escala
            [1.0, 1.0, 1.0, 1.0],    // Color
        );
        
        
    
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
    
            let uniforms = Uniforms::new(
                *position,
                colors[i],
                scales[i],
                orbital_radii[i],
                orbital_speeds[i],
            );
    
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
    
        // Generar pipeline de estrellas
        let star_pipeline = Self::generate_star_pipeline(&device, &config);
    
        // Generar buffer de estrellas
        let (star_buffer, num_stars) = Self::generate_stars(&device);

    
        Self {
            device,
            queue,
            surface,
            config,
            depth_view,
            spheres,
            star_buffer,
            num_stars,
            star_pipeline,
            spaceship,
            spaceship_position: cgmath::Vector3::new(0.0, 0.0, 0.0), // Posición inicial
            spaceship_rotation: cgmath::Vector3::new(0.0, 0.0, 0.0), // Sin rotación inicial

        }
        
    }
    

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder"),
        });

        self.render_stars(&mut encoder, &view);

        // Render pass para las estrellas
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Star Render Pass"),
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
                depth_stencil_attachment: None, // Las estrellas no necesitan profundidad
            });
    
            render_pass.set_pipeline(&self.star_pipeline);
            render_pass.set_vertex_buffer(0, self.star_buffer.slice(..));
            render_pass.draw(0..self.num_stars, 0..1);

        }
    
        // Render pass para los objetos 3D
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Mantiene el contenido previo (las estrellas)
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // Limpia la profundidad para los objetos 3D
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
    
            // Renderizado de las esferas en orden
            let mut render_order: Vec<(usize, f32)> = self.spheres
                .iter()
                .enumerate()
                .map(|(i, sphere)| {
                    // Extraer la posición Z de la matriz model
                    let translation_z = sphere.uniforms.model[3][2];
                    (i, translation_z)
                })
                .collect();
    
            // Ordenar de atrás hacia adelante (z más negativo primero)
            render_order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
            for (index, _) in render_order {
                let sphere = &self.spheres[index];
                render_pass.set_pipeline(&sphere.pipeline);
                render_pass.set_bind_group(0, &sphere.bind_group, &[]);
                render_pass.set_vertex_buffer(0, sphere.vertex_buffer.slice(..));
                render_pass.set_index_buffer(sphere.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..sphere.num_indices, 0, 0..1);
            }
            render_pass.set_pipeline(&self.spaceship.pipeline);
            render_pass.set_bind_group(0, &self.spaceship.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.spaceship.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.spaceship.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.spaceship.num_indices, 0, 0..1);

        }
        
    
        self.queue.submit(std::iter::once(encoder.finish())); // Enviar comandos a la GPU
        output.present(); // Presentar el frame actual en la ventana
    
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

    // Variables para controlar la velocidad y rotación de la nave manualmente
    const MOVE_SPEED: f32 = 0.2; // Velocidad de movimiento
    const ROTATE_SPEED: f32 = 0.05; // Velocidad de rotación

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    // Capturar teclas para mover la nave manualmente
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => {
                        match key {
                            VirtualKeyCode::W => state.spaceship_position.z -= MOVE_SPEED, // Adelante
                            VirtualKeyCode::S => state.spaceship_position.z += MOVE_SPEED, // Atrás
                            VirtualKeyCode::A => state.spaceship_position.x -= MOVE_SPEED, // Izquierda
                            VirtualKeyCode::D => state.spaceship_position.x += MOVE_SPEED, // Derecha
                            VirtualKeyCode::Space => state.spaceship_position.y += MOVE_SPEED, // Subir
                            VirtualKeyCode::LShift => state.spaceship_position.y -= MOVE_SPEED, // Bajar
                            VirtualKeyCode::Left => state.spaceship_rotation.y -= ROTATE_SPEED, // Rotar izquierda
                            VirtualKeyCode::Right => state.spaceship_rotation.y += ROTATE_SPEED, // Rotar derecha
                            VirtualKeyCode::Up => state.spaceship_rotation.x -= ROTATE_SPEED, // Rotar arriba
                            VirtualKeyCode::Down => state.spaceship_rotation.x += ROTATE_SPEED, // Rotar abajo
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                current_time += 0.016; // Incrementa el tiempo ~60 FPS

                // Actualiza la posición y rotación de los planetas
                for sphere in &mut state.spheres {
                    let mut uniforms = sphere.uniforms;
                    uniforms.time = current_time;

                    if uniforms.orbital_speed > 0.0 {
                        let angle = current_time * uniforms.orbital_speed;
                        let x = uniforms.orbital_radius * angle.cos();
                        let z = uniforms.orbital_radius * angle.sin();

                        let translation = cgmath::Matrix4::from_translation(cgmath::Vector3::new(x, 0.0, z));
                        let rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(angle));
                        let scale = cgmath::Matrix4::from_scale(uniforms.model[0][0]);

                        uniforms.model = (translation * rotation * scale).into();
                    }

                    state.queue.write_buffer(
                        &sphere.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[uniforms]),
                    );
                }

                // Actualiza la nave espacial manualmente
                let translation = cgmath::Matrix4::from_translation(state.spaceship_position);
                let rotation = cgmath::Matrix4::from_angle_y(cgmath::Rad(state.spaceship_rotation.y))
                    * cgmath::Matrix4::from_angle_x(cgmath::Rad(state.spaceship_rotation.x))
                    * cgmath::Matrix4::from_angle_z(cgmath::Rad(state.spaceship_rotation.z));
                let scale = cgmath::Matrix4::from_scale(0.2);

                state.spaceship.uniforms.model = (translation * rotation * scale).into();
                state.queue.write_buffer(
                    &state.spaceship.uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[state.spaceship.uniforms]),
                );

                // Solicita un redibujo
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                state.render().unwrap();
            }
            _ => {}
        }
    });
}

