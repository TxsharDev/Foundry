use crate::scene::*;
use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight,
};

pub struct TextEngine {
    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub buffers: Vec<(NodeId, Buffer)>,
    pub viewport: Viewport,
}

impl TextEngine {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let glyph_cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &glyph_cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(device, &glyph_cache);

        Self {
            font_system,
            cache,
            atlas,
            text_renderer,
            buffers: Vec::new(),
            viewport,
        }
    }

    pub fn prepare(
        &mut self,
        scene: &SceneGraph,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        self.buffers.clear();

        if let Some(root) = scene.root {
            self.collect_text_nodes(scene, root);
        }

        self.viewport.update(
            queue,
            Resolution {
                width: viewport_width,
                height: viewport_height,
            },
        );

        let text_areas: Vec<TextArea> = self
            .buffers
            .iter()
            .map(|(node_id, buffer)| {
                let node = scene.get(*node_id);
                let color = &node.style.color;
                let r = (color.r * 255.0) as u8;
                let g = (color.g * 255.0) as u8;
                let b = (color.b * 255.0) as u8;
                let a = (color.a * 255.0) as u8;

                TextArea {
                    buffer,
                    left: node.layout.x,
                    top: node.layout.y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: viewport_width as i32,
                        bottom: viewport_height as i32,
                    },
                    default_color: GlyphonColor::rgba(r, g, b, a),
                    custom_glyphs: &[],
                }
            })
            .collect();

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.cache,
            )
            .ok();
    }

    pub fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.text_renderer
            .render(&self.atlas, &self.viewport, pass)
            .ok();
    }

    fn collect_text_nodes(&mut self, scene: &SceneGraph, node_id: NodeId) {
        let node = scene.get(node_id);

        if node.style.display == Display::None {
            return;
        }

        if node.kind == ElementKind::Text {
            if let Some(text) = &node.text_content {
                let parent_style = node
                    .parent
                    .map(|p| &scene.get(p).style)
                    .unwrap_or(&node.style);

                let font_size = parent_style.font_size;
                let line_height = font_size * parent_style.line_height;

                let mut buffer =
                    Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));

                let weight = if parent_style.font_weight >= 700 {
                    Weight::BOLD
                } else if parent_style.font_weight >= 500 {
                    Weight::MEDIUM
                } else {
                    Weight::NORMAL
                };

                let family = if parent_style.font_family.is_empty() {
                    Family::SansSerif
                } else {
                    Family::Name(&parent_style.font_family)
                };

                let attrs = Attrs::new().family(family).weight(weight);

                // Never constrain single-line text width. Let glyphon render
                // the full text without wrapping. Overflow is better than clipping.
                buffer.set_size(&mut self.font_system, None, None);
                buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
                buffer.shape_until_scroll(&mut self.font_system, false);

                self.buffers.push((node_id, buffer));
            }
        }

        let children: Vec<NodeId> = node.children.clone();
        for child_id in children {
            self.collect_text_nodes(scene, child_id);
        }
    }
}
