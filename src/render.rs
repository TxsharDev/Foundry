use wgpu::util::DeviceExt;

use crate::scene::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 2],
    color: [f32; 4],
    border_color: [f32; 4],
    rect: [f32; 4], // x, y, width, height in pixels
    border_radius: [f32; 4],
    border_width: f32,
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Viewport {
    size: [f32; 2],
    _pad: [f32; 2],
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    quad_pipeline: wgpu::RenderPipeline,
    viewport_buffer: wgpu::Buffer,
    viewport_bind_group: wgpu::BindGroup,
    viewport: [f32; 2],
    pub format: wgpu::TextureFormat,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("foundry"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::default(),
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Viewport uniform buffer
        let viewport_data = Viewport {
            size: [size.width as f32, size.height as f32],
            _pad: [0.0; 2],
        };
        let viewport_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("viewport_uniform"),
            contents: bytemuck::cast_slice(&[viewport_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let viewport_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad_shader"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            ..Default::default()
        });

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<QuadVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,  // position
                        1 => Float32x4,  // color
                        2 => Float32x4,  // border_color
                        3 => Float32x4,  // rect
                        4 => Float32x4,  // border_radius
                        5 => Float32,    // border_width
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            device,
            queue,
            surface,
            config,
            quad_pipeline,
            viewport_buffer,
            viewport_bind_group,
            viewport: [size.width as f32, size.height as f32],
            format,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.viewport = [width as f32, height as f32];

            let viewport_data = Viewport {
                size: self.viewport,
                _pad: [0.0; 2],
            };
            self.queue.write_buffer(
                &self.viewport_buffer,
                0,
                bytemuck::cast_slice(&[viewport_data]),
            );
        }
    }

    pub fn viewport_size(&self) -> (f32, f32) {
        (self.viewport[0], self.viewport[1])
    }

    pub fn render(
        &mut self,
        scene: &SceneGraph,
        mut text_engine: Option<&mut crate::text::TextEngine>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            other => {
                return Err(format!("surface error: {:?}", other).into());
            }
        };
        let view = output.texture.create_view(&Default::default());

        if let Some(te) = text_engine.as_mut() {
            te.prepare(
                scene,
                &self.device,
                &self.queue,
                self.config.width,
                self.config.height,
            );
        }

        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        if let Some(root) = scene.root {
            self.collect_quads(scene, root, &mut vertices, &mut indices, 0.0, 0.0);
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        // Use root node's background color as clear color (or white fallback)
        let clear_color = scene
            .root
            .map(|r| {
                let bg = &scene.get(r).style.background_color;
                if bg.a > 0.0 {
                    return wgpu::Color {
                        r: bg.r as f64,
                        g: bg.g as f64,
                        b: bg.b as f64,
                        a: bg.a as f64,
                    };
                }
                // Check body (first child of root)
                for &child in &scene.get(r).children {
                    let cbg = &scene.get(child).style.background_color;
                    if cbg.a > 0.0 {
                        return wgpu::Color {
                            r: cbg.r as f64,
                            g: cbg.g as f64,
                            b: cbg.b as f64,
                            a: cbg.a as f64,
                        };
                    }
                }
                wgpu::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }
            })
            .unwrap_or(wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if !indices.is_empty() {
                pass.set_pipeline(&self.quad_pipeline);
                pass.set_bind_group(0, &self.viewport_bind_group, &[]);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }

            if let Some(te) = text_engine.as_ref() {
                te.render(&mut pass);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn collect_quads(
        &self,
        scene: &SceneGraph,
        node_id: NodeId,
        vertices: &mut Vec<QuadVertex>,
        indices: &mut Vec<u32>,
        scroll_x: f32,
        scroll_y: f32,
    ) {
        let node = scene.get(node_id);

        if node.style.display == Display::None {
            return;
        }

        let layout = &node.layout;
        let x = layout.x - scroll_x;
        let y = layout.y - scroll_y;
        let w = layout.width;
        let h = layout.height;

        if w <= 0.0 || h <= 0.0 {
            let sx = scroll_x + node.scroll_offset.0;
            let sy = scroll_y + node.scroll_offset.1;
            for &child_id in &node.children {
                self.collect_quads(scene, child_id, vertices, indices, sx, sy);
            }
            return;
        }

        let bg = node.style.background_color;
        let has_bg = bg.a > 0.0;
        let has_border = node.style.border_width.iter().any(|&w| w > 0.0);

        if has_bg || has_border {
            let base_idx = vertices.len() as u32;

            let color = [bg.r, bg.g, bg.b, bg.a * node.style.opacity];
            let bc = node.style.border_color;
            let border_color = [bc.r, bc.g, bc.b, bc.a];
            let rect = [x, y, w, h];
            let border_radius = node.style.border_radius;
            let border_width = node.style.border_width[0];

            let corners = [[x, y], [x + w, y], [x + w, y + h], [x, y + h]];

            for pos in corners {
                vertices.push(QuadVertex {
                    position: pos,
                    color,
                    border_color,
                    rect,
                    border_radius,
                    border_width,
                    _padding: [0.0; 3],
                });
            }

            indices.extend_from_slice(&[
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ]);
        }

        let sx = scroll_x + node.scroll_offset.0;
        let sy = scroll_y + node.scroll_offset.1;
        for &child_id in &node.children {
            self.collect_quads(scene, child_id, vertices, indices, sx, sy);
        }
    }
}

const QUAD_SHADER: &str = r#"
struct Viewport {
    size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> viewport: Viewport;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) border_color: vec4<f32>,
    @location(3) rect: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_width: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) pixel_pos: vec2<f32>,
    @location(3) rect: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_width: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let x = (in.position.x / viewport.size.x) * 2.0 - 1.0;
    let y = 1.0 - (in.position.y / viewport.size.y) * 2.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.color = in.color;
    out.border_color = in.border_color;
    out.pixel_pos = in.position;
    out.rect = in.rect;
    out.border_radius = in.border_radius;
    out.border_width = in.border_width;
    return out;
}

fn rounded_rect_sdf(pixel: vec2<f32>, rect: vec4<f32>, radius: vec4<f32>) -> f32 {
    let center = vec2<f32>(rect.x + rect.z * 0.5, rect.y + rect.w * 0.5);
    let half_size = vec2<f32>(rect.z * 0.5, rect.w * 0.5);
    let p = pixel - center;

    var r: f32;
    if p.x < 0.0 {
        if p.y < 0.0 { r = radius.x; }
        else { r = radius.w; }
    } else {
        if p.y < 0.0 { r = radius.y; }
        else { r = radius.z; }
    }

    let q = abs(p) - half_size + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = rounded_rect_sdf(in.pixel_pos, in.rect, in.border_radius);

    let aa = 1.0;
    let alpha = 1.0 - smoothstep(-aa, aa, d);

    if alpha < 0.001 {
        discard;
    }

    if in.border_width > 0.0 {
        let inner_d = rounded_rect_sdf(
            in.pixel_pos,
            vec4<f32>(
                in.rect.x + in.border_width,
                in.rect.y + in.border_width,
                in.rect.z - in.border_width * 2.0,
                in.rect.w - in.border_width * 2.0,
            ),
            max(in.border_radius - vec4<f32>(in.border_width), vec4<f32>(0.0)),
        );

        let border_alpha = smoothstep(-aa, aa, inner_d);
        let color = mix(in.color, in.border_color, border_alpha);
        return vec4<f32>(color.rgb, color.a * alpha);
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#;
