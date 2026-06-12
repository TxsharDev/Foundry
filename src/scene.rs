use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Div,
    Span,
    P,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Button,
    Input,
    Img,
    A,
    Ul,
    Ol,
    Li,
    Body,
    Html,
    Text,
    Unknown,
}

impl ElementKind {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "div" => Self::Div,
            "span" => Self::Span,
            "p" => Self::P,
            "h1" => Self::H1,
            "h2" => Self::H2,
            "h3" => Self::H3,
            "h4" => Self::H4,
            "h5" => Self::H5,
            "h6" => Self::H6,
            "button" => Self::Button,
            "input" => Self::Input,
            "img" => Self::Img,
            "a" => Self::A,
            "ul" => Self::Ul,
            "ol" => Self::Ol,
            "li" => Self::Li,
            "body" => Self::Body,
            "html" => Self::Html,
            _ => Self::Unknown,
        }
    }

    pub fn is_inline(&self) -> bool {
        matches!(self, Self::Span | Self::A | Self::Text)
    }

    pub fn is_block(&self) -> bool {
        !self.is_inline() && !matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedStyle {
    // Box model (px)
    pub margin: [f32; 4], // top, right, bottom, left
    pub padding: [f32; 4],
    pub border_width: [f32; 4],

    // Sizing
    pub width: SizeValue,
    pub height: SizeValue,
    pub min_width: SizeValue,
    pub min_height: SizeValue,
    pub max_width: SizeValue,
    pub max_height: SizeValue,

    // Flexbox
    pub display: Display,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub gap: f32,

    // Position
    pub position: Position,
    pub top: SizeValue,
    pub left: SizeValue,
    pub right: SizeValue,
    pub bottom: SizeValue,

    // Visual
    pub background_color: Color,
    pub color: Color,
    pub border_color: Color,
    pub border_radius: [f32; 4], // top-left, top-right, bottom-right, bottom-left
    pub opacity: f32,

    // Text
    pub font_size: f32,
    pub font_weight: u16,
    pub font_family: String,
    pub text_align: TextAlign,
    pub line_height: f32,

    // Overflow
    pub overflow: Overflow,

    // Z-index
    pub z_index: i32,

    // Transitions
    pub transition_duration: f32,
    pub transition_property: TransitionProperty,

    // Animation
    pub animation_name: String,
    pub animation_duration: f32,
    pub animation_delay: f32,
    pub animation_iteration_count: f32,
    pub animation_direction: AnimationDirection,
}

impl Default for ResolvedStyle {
    fn default() -> Self {
        Self {
            margin: [0.0; 4],
            padding: [0.0; 4],
            border_width: [0.0; 4],
            width: SizeValue::Auto,
            height: SizeValue::Auto,
            min_width: SizeValue::Auto,
            min_height: SizeValue::Auto,
            max_width: SizeValue::Auto,
            max_height: SizeValue::Auto,
            display: Display::Block,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            gap: 0.0,
            position: Position::Relative,
            top: SizeValue::Auto,
            left: SizeValue::Auto,
            right: SizeValue::Auto,
            bottom: SizeValue::Auto,
            background_color: Color::TRANSPARENT,
            color: Color::BLACK,
            border_color: Color::BLACK,
            border_radius: [0.0; 4],
            opacity: 1.0,
            font_size: 16.0,
            font_weight: 400,
            font_family: String::new(),
            text_align: TextAlign::Left,
            line_height: 1.2,
            overflow: Overflow::Visible,
            z_index: 0,
            transition_duration: 0.0,
            transition_property: TransitionProperty::None,
            animation_name: String::new(),
            animation_duration: 0.0,
            animation_delay: 0.0,
            animation_iteration_count: 1.0,
            animation_direction: AnimationDirection::Normal,
        }
    }
}

impl ResolvedStyle {
    /// Interpolate between self and target by t (0.0 = self, 1.0 = target)
    pub fn lerp(&self, target: &ResolvedStyle, t: f32) -> ResolvedStyle {
        let mut result = target.clone();
        let t = t.clamp(0.0, 1.0);
        let inv = 1.0 - t;

        result.background_color = Color {
            r: self.background_color.r * inv + target.background_color.r * t,
            g: self.background_color.g * inv + target.background_color.g * t,
            b: self.background_color.b * inv + target.background_color.b * t,
            a: self.background_color.a * inv + target.background_color.a * t,
        };
        result.color = Color {
            r: self.color.r * inv + target.color.r * t,
            g: self.color.g * inv + target.color.g * t,
            b: self.color.b * inv + target.color.b * t,
            a: self.color.a * inv + target.color.a * t,
        };
        result.border_color = Color {
            r: self.border_color.r * inv + target.border_color.r * t,
            g: self.border_color.g * inv + target.border_color.g * t,
            b: self.border_color.b * inv + target.border_color.b * t,
            a: self.border_color.a * inv + target.border_color.a * t,
        };

        result.opacity = self.opacity * inv + target.opacity * t;
        result.font_size = self.font_size * inv + target.font_size * t;
        result.border_radius = [
            self.border_radius[0] * inv + target.border_radius[0] * t,
            self.border_radius[1] * inv + target.border_radius[1] * t,
            self.border_radius[2] * inv + target.border_radius[2] * t,
            self.border_radius[3] * inv + target.border_radius[3] * t,
        ];
        result.border_width = [
            self.border_width[0] * inv + target.border_width[0] * t,
            self.border_width[1] * inv + target.border_width[1] * t,
            self.border_width[2] * inv + target.border_width[2] * t,
            self.border_width[3] * inv + target.border_width[3] * t,
        ];
        result.padding = [
            self.padding[0] * inv + target.padding[0] * t,
            self.padding[1] * inv + target.padding[1] * t,
            self.padding[2] * inv + target.padding[2] * t,
            self.padding[3] * inv + target.padding[3] * t,
        ];
        result.margin = [
            self.margin[0] * inv + target.margin[0] * t,
            self.margin[1] * inv + target.margin[1] * t,
            self.margin[2] * inv + target.margin[2] * t,
            self.margin[3] * inv + target.margin[3] * t,
        ];

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeValue {
    Px(f32),
    Percent(f32),
    Em(f32),
    Rem(f32),
    Vh(f32),
    Vw(f32),
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Block,
    Flex,
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignItems {
    #[default]
    Stretch,
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionProperty {
    #[default]
    None,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn from_rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for LayoutRect {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: NodeId,
    pub kind: ElementKind,
    pub tag: String,
    pub text_content: Option<String>,
    pub attributes: HashMap<String, String>,
    pub classes: Vec<String>,
    pub element_id: Option<String>,
    pub style: ResolvedStyle,
    pub layout: LayoutRect,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub dirty: bool,
    pub scroll_offset: (f32, f32),
    pub content_height: f32,
    pub event_handlers: HashMap<String, String>, // event name -> JS code
    pub hover_style: Option<ResolvedStyle>,      // :hover overrides
    pub base_style: Option<ResolvedStyle>,       // original style (before hover)
    pub is_hovered: bool,
    pub image_src: Option<String>,
    // Transition animation state
    pub transition_start: Option<std::time::Instant>,
    pub transition_from: Option<ResolvedStyle>,
    pub transition_to: Option<ResolvedStyle>,
    // CSS animation state
    pub animation: Option<AnimationState>,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
    pub percent: f32, // 0.0 = from, 1.0 = to
    pub style: ResolvedStyle,
}

#[derive(Debug, Clone)]
pub struct KeyframeAnimation {
    pub name: String,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub animation_name: String,
    pub duration: f32,        // seconds
    pub delay: f32,           // seconds
    pub iteration_count: f32, // f32::INFINITY for infinite
    pub direction: AnimationDirection,
    pub start_time: Option<std::time::Instant>,
    pub current_iteration: f32,
    pub keyframes: Vec<Keyframe>,
}

impl SceneNode {
    pub fn new(id: NodeId, kind: ElementKind, tag: String) -> Self {
        let mut style = ResolvedStyle::default();
        // Set default display based on element kind
        if kind.is_inline() {
            style.display = Display::Inline;
        }
        // Heading font sizes
        match kind {
            ElementKind::H1 => {
                style.font_size = 32.0;
                style.font_weight = 700;
            }
            ElementKind::H2 => {
                style.font_size = 24.0;
                style.font_weight = 700;
            }
            ElementKind::H3 => {
                style.font_size = 20.0;
                style.font_weight = 700;
            }
            ElementKind::H4 => {
                style.font_size = 16.0;
                style.font_weight = 700;
            }
            ElementKind::H5 => {
                style.font_size = 14.0;
                style.font_weight = 700;
            }
            ElementKind::H6 => {
                style.font_size = 12.0;
                style.font_weight = 700;
            }
            _ => {}
        }

        Self {
            id,
            kind,
            tag,
            text_content: None,
            attributes: HashMap::new(),
            classes: Vec::new(),
            element_id: None,
            style,
            layout: LayoutRect::default(),
            children: Vec::new(),
            parent: None,
            dirty: true,
            scroll_offset: (0.0, 0.0),
            content_height: 0.0,
            event_handlers: HashMap::new(),
            hover_style: None,
            base_style: None,
            is_hovered: false,
            image_src: None,
            transition_start: None,
            transition_from: None,
            transition_to: None,
            animation: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct SceneGraph {
    pub nodes: Vec<SceneNode>,
    pub root: Option<NodeId>,
    pub keyframes: Vec<KeyframeAnimation>,
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            keyframes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, kind: ElementKind, tag: String) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(SceneNode::new(id, kind, tag));
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        self.nodes[parent.0].children.push(child);
        self.nodes[child.0].parent = Some(parent);
    }

    pub fn get(&self, id: NodeId) -> &SceneNode {
        &self.nodes[id.0]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut SceneNode {
        &mut self.nodes[id.0]
    }

    pub fn mark_dirty_recursive(&mut self, id: NodeId) {
        self.nodes[id.0].dirty = true;
        let children: Vec<NodeId> = self.nodes[id.0].children.clone();
        for child in children {
            self.mark_dirty_recursive(child);
        }
    }

    pub fn find_by_element_id(&self, element_id: &str) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|n| n.element_id.as_deref() == Some(element_id))
            .map(|n| n.id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Tick all animations. Returns true if any animation is active (needs redraw).
    pub fn tick_animations(&mut self) -> bool {
        let now = std::time::Instant::now();
        let mut any_active = false;

        for node in &mut self.nodes {
            // Tick CSS transitions
            if let (Some(start), Some(from), Some(to)) = (
                node.transition_start,
                node.transition_from.as_ref(),
                node.transition_to.as_ref(),
            ) {
                let duration = node
                    .style
                    .transition_duration
                    .max(from.transition_duration.max(to.transition_duration));
                if duration <= 0.0 {
                    node.style = to.clone();
                    node.transition_start = None;
                    node.transition_from = None;
                    node.transition_to = None;
                    node.dirty = true;
                    continue;
                }
                let elapsed = now.duration_since(start).as_secs_f32();
                let t = (elapsed / duration).clamp(0.0, 1.0);
                node.style = from.lerp(to, t);
                node.style.transition_duration = to.transition_duration;
                node.style.transition_property = to.transition_property;
                node.dirty = true;
                any_active = true;
                if t >= 1.0 {
                    node.style = to.clone();
                    node.transition_start = None;
                    node.transition_from = None;
                    node.transition_to = None;
                    if !node.is_hovered {
                        if let Some(base) = &node.base_style {
                            node.style = base.clone();
                        }
                    }
                }
            }

            // Tick CSS keyframe animations
            if let Some(anim) = &mut node.animation {
                if anim.keyframes.len() < 2 || anim.duration <= 0.0 {
                    continue;
                }
                if anim.start_time.is_none() {
                    anim.start_time = Some(now);
                }
                let start = anim.start_time.unwrap();
                let elapsed = now.duration_since(start).as_secs_f32() - anim.delay;
                if elapsed < 0.0 {
                    any_active = true;
                    continue; // Still in delay
                }

                let cycle_time = elapsed % anim.duration;
                let iteration = (elapsed / anim.duration).floor();

                if iteration >= anim.iteration_count && !anim.iteration_count.is_infinite() {
                    // Animation finished — apply final keyframe
                    if let Some(last) = anim.keyframes.last() {
                        apply_keyframe_style(&mut node.style, &last.style);
                    }
                    node.animation = None;
                    node.dirty = true;
                    continue;
                }

                let mut progress = cycle_time / anim.duration;
                match anim.direction {
                    AnimationDirection::Reverse => progress = 1.0 - progress,
                    AnimationDirection::Alternate => {
                        if iteration as u32 % 2 == 1 {
                            progress = 1.0 - progress;
                        }
                    }
                    AnimationDirection::Normal => {}
                }

                // Find the two keyframes to interpolate between
                let kfs = &anim.keyframes;
                let mut from_idx = 0;
                let mut to_idx = kfs.len() - 1;
                for j in 0..kfs.len() - 1 {
                    if progress >= kfs[j].percent && progress <= kfs[j + 1].percent {
                        from_idx = j;
                        to_idx = j + 1;
                        break;
                    }
                }

                let from_pct = kfs[from_idx].percent;
                let to_pct = kfs[to_idx].percent;
                let local_t = if (to_pct - from_pct).abs() > 0.001 {
                    (progress - from_pct) / (to_pct - from_pct)
                } else {
                    1.0
                };

                let interpolated = kfs[from_idx].style.lerp(&kfs[to_idx].style, local_t);
                apply_keyframe_style(&mut node.style, &interpolated);
                node.dirty = true;
                any_active = true;
            }
        }

        any_active
    }
}

fn apply_keyframe_style(target: &mut ResolvedStyle, kf: &ResolvedStyle) {
    target.background_color = kf.background_color;
    target.color = kf.color;
    target.border_color = kf.border_color;
    target.opacity = kf.opacity;
    target.border_radius = kf.border_radius;
    target.padding = kf.padding;
    target.margin = kf.margin;
}
