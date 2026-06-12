use crate::scene::*;

pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();

    // Named colors
    match s.to_lowercase().as_str() {
        "transparent" => return Some(Color::TRANSPARENT),
        "black" => return Some(Color::BLACK),
        "white" => return Some(Color::WHITE),
        "red" => return Some(Color::from_rgba(255, 0, 0, 1.0)),
        "green" => return Some(Color::from_rgba(0, 128, 0, 1.0)),
        "blue" => return Some(Color::from_rgba(0, 0, 255, 1.0)),
        "yellow" => return Some(Color::from_rgba(255, 255, 0, 1.0)),
        "orange" => return Some(Color::from_rgba(255, 165, 0, 1.0)),
        "purple" => return Some(Color::from_rgba(128, 0, 128, 1.0)),
        "pink" => return Some(Color::from_rgba(255, 192, 203, 1.0)),
        "gray" | "grey" => return Some(Color::from_rgba(128, 128, 128, 1.0)),
        "lightgray" | "lightgrey" => return Some(Color::from_rgba(211, 211, 211, 1.0)),
        "darkgray" | "darkgrey" => return Some(Color::from_rgba(169, 169, 169, 1.0)),
        "cyan" => return Some(Color::from_rgba(0, 255, 255, 1.0)),
        "magenta" => return Some(Color::from_rgba(255, 0, 255, 1.0)),
        "brown" => return Some(Color::from_rgba(165, 42, 42, 1.0)),
        "navy" => return Some(Color::from_rgba(0, 0, 128, 1.0)),
        "teal" => return Some(Color::from_rgba(0, 128, 128, 1.0)),
        "coral" => return Some(Color::from_rgba(255, 127, 80, 1.0)),
        "salmon" => return Some(Color::from_rgba(250, 128, 114, 1.0)),
        "gold" => return Some(Color::from_rgba(255, 215, 0, 1.0)),
        "silver" => return Some(Color::from_rgba(192, 192, 192, 1.0)),
        "olive" => return Some(Color::from_rgba(128, 128, 0, 1.0)),
        "maroon" => return Some(Color::from_rgba(128, 0, 0, 1.0)),
        "lime" => return Some(Color::from_rgba(0, 255, 0, 1.0)),
        "aqua" => return Some(Color::from_rgba(0, 255, 255, 1.0)),
        "indigo" => return Some(Color::from_rgba(75, 0, 130, 1.0)),
        "violet" => return Some(Color::from_rgba(238, 130, 238, 1.0)),
        "tomato" => return Some(Color::from_rgba(255, 99, 71, 1.0)),
        "skyblue" => return Some(Color::from_rgba(135, 206, 235, 1.0)),
        "steelblue" => return Some(Color::from_rgba(70, 130, 180, 1.0)),
        "slategray" | "slategrey" => return Some(Color::from_rgba(112, 128, 144, 1.0)),
        "whitesmoke" => return Some(Color::from_rgba(245, 245, 245, 1.0)),
        "aliceblue" => return Some(Color::from_rgba(240, 248, 255, 1.0)),
        "dodgerblue" => return Some(Color::from_rgba(30, 144, 255, 1.0)),
        "crimson" => return Some(Color::from_rgba(220, 20, 60, 1.0)),
        "darkblue" => return Some(Color::from_rgba(0, 0, 139, 1.0)),
        "darkgreen" => return Some(Color::from_rgba(0, 100, 0, 1.0)),
        "darkred" => return Some(Color::from_rgba(139, 0, 0, 1.0)),
        _ => {}
    }

    // #RGB, #RRGGBB, #RRGGBBAA
    if let Some(hex) = s.strip_prefix('#') {
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                return Some(Color::from_rgba(r, g, b, 1.0));
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::from_rgba(r, g, b, 1.0));
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color::from_rgba(r, g, b, a as f32 / 255.0));
            }
            _ => return None,
        }
    }

    // rgb(r, g, b) and rgba(r, g, b, a)
    if s.starts_with("rgb") {
        let inner = s
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')');
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            let a = if parts.len() == 4 {
                parts[3].trim().parse::<f32>().unwrap_or(1.0)
            } else {
                1.0
            };
            return Some(Color::from_rgba(r, g, b, a));
        }
    }

    None
}

pub fn parse_length(s: &str) -> Option<SizeValue> {
    let s = s.trim();
    if s == "auto" {
        return Some(SizeValue::Auto);
    }
    if s == "0" {
        return Some(SizeValue::Px(0.0));
    }
    if let Some(v) = s.strip_suffix("px") {
        return v.trim().parse::<f32>().ok().map(SizeValue::Px);
    }
    if let Some(v) = s.strip_suffix('%') {
        return v.trim().parse::<f32>().ok().map(SizeValue::Percent);
    }
    if let Some(v) = s.strip_suffix("rem") {
        return v.trim().parse::<f32>().ok().map(SizeValue::Rem);
    }
    if let Some(v) = s.strip_suffix("em") {
        return v.trim().parse::<f32>().ok().map(SizeValue::Em);
    }
    if let Some(v) = s.strip_suffix("vh") {
        return v.trim().parse::<f32>().ok().map(SizeValue::Vh);
    }
    if let Some(v) = s.strip_suffix("vw") {
        return v.trim().parse::<f32>().ok().map(SizeValue::Vw);
    }
    // Plain number -> px
    s.parse::<f32>().ok().map(SizeValue::Px)
}

pub fn apply_property(style: &mut ResolvedStyle, property: &str, value: &str) {
    let prop = property.trim().to_lowercase();
    let val = value.trim();

    match prop.as_str() {
        // Display
        "display" => {
            style.display = match val {
                "flex" => Display::Flex,
                "block" => Display::Block,
                "inline" => Display::Inline,
                "none" => Display::None,
                _ => style.display,
            };
        }

        // Flexbox
        "flex-direction" => {
            style.flex_direction = match val {
                "row" => FlexDirection::Row,
                "column" => FlexDirection::Column,
                "row-reverse" => FlexDirection::RowReverse,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => style.flex_direction,
            };
        }
        "justify-content" => {
            style.justify_content = match val {
                "flex-start" | "start" => JustifyContent::Start,
                "center" => JustifyContent::Center,
                "flex-end" | "end" => JustifyContent::End,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => style.justify_content,
            };
        }
        "align-items" => {
            style.align_items = match val {
                "stretch" => AlignItems::Stretch,
                "flex-start" | "start" => AlignItems::Start,
                "center" => AlignItems::Center,
                "flex-end" | "end" => AlignItems::End,
                _ => style.align_items,
            };
        }
        "flex-wrap" => {
            style.flex_wrap = match val {
                "wrap" => FlexWrap::Wrap,
                "nowrap" => FlexWrap::NoWrap,
                _ => style.flex_wrap,
            };
        }
        "flex-grow" => {
            if let Ok(v) = val.parse::<f32>() {
                style.flex_grow = v;
            }
        }
        "flex-shrink" => {
            if let Ok(v) = val.parse::<f32>() {
                style.flex_shrink = v;
            }
        }
        "gap" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.gap = v;
            }
        }

        // Sizing
        "width" => {
            if let Some(v) = parse_length(val) {
                style.width = v;
            }
        }
        "height" => {
            if let Some(v) = parse_length(val) {
                style.height = v;
            }
        }
        "min-width" => {
            if let Some(v) = parse_length(val) {
                style.min_width = v;
            }
        }
        "min-height" => {
            if let Some(v) = parse_length(val) {
                style.min_height = v;
            }
        }
        "max-width" => {
            if let Some(v) = parse_length(val) {
                style.max_width = v;
            }
        }
        "max-height" => {
            if let Some(v) = parse_length(val) {
                style.max_height = v;
            }
        }

        // Margin
        "margin" => {
            let parts = parse_box_shorthand(val);
            style.margin = parts;
        }
        "margin-top" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.margin[0] = v;
            }
        }
        "margin-right" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.margin[1] = v;
            }
        }
        "margin-bottom" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.margin[2] = v;
            }
        }
        "margin-left" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.margin[3] = v;
            }
        }

        // Padding
        "padding" => {
            let parts = parse_box_shorthand(val);
            style.padding = parts;
        }
        "padding-top" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.padding[0] = v;
            }
        }
        "padding-right" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.padding[1] = v;
            }
        }
        "padding-bottom" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.padding[2] = v;
            }
        }
        "padding-left" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.padding[3] = v;
            }
        }

        // Border
        "border" => {
            // border: 1px solid #000
            let parts: Vec<&str> = val.split_whitespace().collect();
            if let Some(first) = parts.first() {
                if let Some(SizeValue::Px(w)) = parse_length(first) {
                    style.border_width = [w; 4];
                }
            }
            if let Some(color_str) = parts.last() {
                if let Some(c) = parse_color(color_str) {
                    style.border_color = c;
                }
            }
        }
        "border-width" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.border_width = [v; 4];
            }
        }
        "border-color" => {
            if let Some(c) = parse_color(val) {
                style.border_color = c;
            }
        }
        "border-radius" => {
            let parts: Vec<&str> = val.split_whitespace().collect();
            match parts.len() {
                1 => {
                    if let Some(SizeValue::Px(v)) = parse_length(parts[0]) {
                        style.border_radius = [v; 4];
                    }
                }
                4 => {
                    for (i, p) in parts.iter().enumerate() {
                        if let Some(SizeValue::Px(v)) = parse_length(p) {
                            style.border_radius[i] = v;
                        }
                    }
                }
                _ => {}
            }
        }

        // Position
        "position" => {
            style.position = match val {
                "relative" => Position::Relative,
                "absolute" => Position::Absolute,
                "fixed" => Position::Fixed,
                _ => style.position,
            };
        }
        "top" => {
            if let Some(v) = parse_length(val) {
                style.top = v;
            }
        }
        "left" => {
            if let Some(v) = parse_length(val) {
                style.left = v;
            }
        }
        "right" => {
            if let Some(v) = parse_length(val) {
                style.right = v;
            }
        }
        "bottom" => {
            if let Some(v) = parse_length(val) {
                style.bottom = v;
            }
        }

        // Visual
        "background-color" | "background" => {
            if let Some(c) = parse_color(val) {
                style.background_color = c;
            }
        }
        "color" => {
            if let Some(c) = parse_color(val) {
                style.color = c;
            }
        }
        "opacity" => {
            if let Ok(v) = val.parse::<f32>() {
                style.opacity = v;
            }
        }

        // Text
        "font-size" => {
            if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.font_size = v;
            }
        }
        "font-weight" => {
            style.font_weight = match val {
                "bold" => 700,
                "normal" => 400,
                "lighter" => 300,
                "bolder" => 800,
                _ => val.parse::<u16>().unwrap_or(400),
            };
        }
        "font-family" => {
            style.font_family = val.trim_matches(|c| c == '\'' || c == '"').to_string();
        }
        "text-align" => {
            style.text_align = match val {
                "left" => TextAlign::Left,
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                _ => style.text_align,
            };
        }
        "line-height" => {
            if let Ok(v) = val.parse::<f32>() {
                style.line_height = v;
            } else if let Some(SizeValue::Px(v)) = parse_length(val) {
                style.line_height = v / style.font_size;
            }
        }

        // Overflow
        "overflow" => {
            style.overflow = match val {
                "hidden" => Overflow::Hidden,
                "scroll" | "auto" => Overflow::Scroll,
                _ => Overflow::Visible,
            };
        }

        // Z-index
        "z-index" => {
            if let Ok(v) = val.parse::<i32>() {
                style.z_index = v;
            }
        }

        // Transitions
        "transition" => {
            // transition: all 0.3s, transition: background-color 0.2s
            let parts: Vec<&str> = val.split_whitespace().collect();
            if parts.len() >= 2 {
                style.transition_property = match parts[0] {
                    "all" => crate::scene::TransitionProperty::All,
                    _ => crate::scene::TransitionProperty::All, // treat any property as "all" for v0.1
                };
                // Parse duration
                let dur = parts[1];
                if let Some(s) = dur.strip_suffix('s') {
                    if let Some(ms) = s.strip_suffix('m') {
                        style.transition_duration = ms.parse::<f32>().unwrap_or(0.0) / 1000.0;
                    } else {
                        style.transition_duration = s.parse::<f32>().unwrap_or(0.0);
                    }
                }
            }
        }
        "transition-duration" => {
            let dur = val.trim();
            if let Some(s) = dur.strip_suffix('s') {
                if let Some(ms) = s.strip_suffix('m') {
                    style.transition_duration = ms.parse::<f32>().unwrap_or(0.0) / 1000.0;
                } else {
                    style.transition_duration = s.parse::<f32>().unwrap_or(0.0);
                }
            }
        }

        // Animation
        "animation" => {
            // animation: name duration [delay] [iteration-count] [direction]
            // e.g. "fadeIn 0.5s", "pulse 2s infinite", "slide 1s 0.2s infinite alternate"
            let parts: Vec<&str> = val.split_whitespace().collect();
            if !parts.is_empty() {
                style.animation_name = parts[0].to_string();
            }
            if parts.len() >= 2 {
                style.animation_duration = parse_time(parts[1]);
            }
            // Check remaining parts for delay, iteration, direction
            for &part in parts.iter().skip(2) {
                if part == "infinite" {
                    style.animation_iteration_count = f32::INFINITY;
                } else if part == "alternate" {
                    style.animation_direction = crate::scene::AnimationDirection::Alternate;
                } else if part == "reverse" {
                    style.animation_direction = crate::scene::AnimationDirection::Reverse;
                } else if part.ends_with('s') {
                    // Could be delay
                    let t = parse_time(part);
                    if t > 0.0 {
                        style.animation_delay = t;
                    }
                } else if let Ok(n) = part.parse::<f32>() {
                    style.animation_iteration_count = n;
                }
            }
        }
        "animation-name" => {
            style.animation_name = val.trim().to_string();
        }
        "animation-duration" => {
            style.animation_duration = parse_time(val.trim());
        }
        "animation-delay" => {
            style.animation_delay = parse_time(val.trim());
        }
        "animation-iteration-count" => {
            if val.trim() == "infinite" {
                style.animation_iteration_count = f32::INFINITY;
            } else if let Ok(n) = val.trim().parse::<f32>() {
                style.animation_iteration_count = n;
            }
        }

        _ => {} // Unsupported property - ignore
    }
}

fn parse_time(s: &str) -> f32 {
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<f32>().unwrap_or(0.0) / 1000.0
    } else if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
    }
}

fn parse_box_shorthand(val: &str) -> [f32; 4] {
    let parts: Vec<f32> = val
        .split_whitespace()
        .map(|p| {
            if let Some(SizeValue::Px(v)) = parse_length(p) {
                v
            } else {
                0.0
            }
        })
        .collect();

    match parts.len() {
        1 => [parts[0]; 4],
        2 => [parts[0], parts[1], parts[0], parts[1]],
        3 => [parts[0], parts[1], parts[2], parts[1]],
        4 => [parts[0], parts[1], parts[2], parts[3]],
        _ => [0.0; 4],
    }
}

pub fn parse_inline_style(style_str: &str, target: &mut ResolvedStyle) {
    for declaration in style_str.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        if let Some((prop, val)) = declaration.split_once(':') {
            apply_property(target, prop, val);
        }
    }
}

// CSS rule: selector + declarations
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: Selector,
    pub declarations: Vec<(String, String)>,
    pub specificity: u32,
}

/// A single selector atom (e.g., "div", ".card", "#main")
#[derive(Debug, Clone)]
pub enum SelectorPart {
    Tag(String),
    Class(String),
    Id(String),
    Universal,
}

/// A segment is one space-separated part of a selector.
/// e.g., "div.btn.active" is ONE segment with parts [Tag("div"), Class("btn"), Class("active")]
/// All parts in a segment must match the SAME element.
#[derive(Debug, Clone)]
pub struct SelectorSegment {
    pub parts: Vec<SelectorPart>,
}

impl SelectorSegment {
    pub fn matches_node(&self, node: &crate::scene::SceneNode) -> bool {
        self.parts.iter().all(|part| match part {
            SelectorPart::Tag(tag) => node.tag == *tag,
            SelectorPart::Class(class) => node.classes.contains(class),
            SelectorPart::Id(id) => node.element_id.as_deref() == Some(id.as_str()),
            SelectorPart::Universal => true,
        })
    }
}

/// A full selector: chain of segments (ancestor > ... > target) + optional :hover
/// e.g. ".card h1:hover" = segments=[Segment([Class("card")]), Segment([Tag("h1")])], hover=true
#[derive(Debug, Clone)]
pub struct Selector {
    pub segments: Vec<SelectorSegment>,
    pub hover: bool,
}

impl Selector {
    pub fn is_hover(&self) -> bool {
        self.hover
    }

    pub fn specificity(&self) -> u32 {
        let mut s: u32 = 0;
        for seg in &self.segments {
            for part in &seg.parts {
                s += match part {
                    SelectorPart::Id(_) => 100,
                    SelectorPart::Class(_) => 10,
                    SelectorPart::Tag(_) => 1,
                    SelectorPart::Universal => 0,
                };
            }
        }
        if self.hover {
            s += 10;
        }
        s
    }

    /// Target-only match, ignores ancestor chain.
    pub fn matches(&self, node: &crate::scene::SceneNode) -> bool {
        if self.segments.is_empty() {
            return false;
        }
        self.segments.last().unwrap().matches_node(node)
    }

    /// Full match: target segment + ancestor chain
    pub fn matches_with_ancestors(
        &self,
        node: &crate::scene::SceneNode,
        scene: &crate::scene::SceneGraph,
    ) -> bool {
        if self.segments.is_empty() {
            return false;
        }
        // Last segment must match target node
        if !self.segments.last().unwrap().matches_node(node) {
            return false;
        }
        if self.segments.len() == 1 {
            return true;
        }
        // Walk up ancestors to match remaining segments
        let ancestor_segs = &self.segments[..self.segments.len() - 1];
        let mut seg_idx = ancestor_segs.len();
        let mut current = node.parent;
        while seg_idx > 0 {
            if let Some(parent_id) = current {
                let parent = scene.get(parent_id);
                if ancestor_segs[seg_idx - 1].matches_node(parent) {
                    seg_idx -= 1;
                }
                current = parent.parent;
            } else {
                return false;
            }
        }
        true
    }
}

pub fn parse_stylesheet(css: &str) -> Vec<CssRule> {
    let (rules, _) = parse_stylesheet_with_keyframes(css);
    rules
}

pub fn parse_stylesheet_with_keyframes(
    css: &str,
) -> (Vec<CssRule>, Vec<crate::scene::KeyframeAnimation>) {
    let mut rules = Vec::new();
    let mut keyframes = Vec::new();
    let mut pos = 0;
    let bytes = css.as_bytes();

    while pos < bytes.len() {
        // Skip whitespace and comments
        while pos < bytes.len() && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        // Skip comments
        if pos + 1 < bytes.len() && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
            if let Some(end) = css[pos..].find("*/") {
                pos += end + 2;
                continue;
            }
            break;
        }

        // Find selector (everything before '{')
        let selector_start = pos;
        while pos < bytes.len() && bytes[pos] != b'{' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        let selector_str = css[selector_start..pos].trim();
        pos += 1; // skip '{'

        // Handle @keyframes blocks
        if selector_str.starts_with("@keyframes") {
            let anim_name = selector_str
                .strip_prefix("@keyframes")
                .unwrap()
                .trim()
                .to_string();
            // Find the end of the @keyframes block (nested braces)
            let kf_start = pos;
            let mut brace_depth = 1;
            while pos < bytes.len() && brace_depth > 0 {
                if bytes[pos] == b'{' {
                    brace_depth += 1;
                } else if bytes[pos] == b'}' {
                    brace_depth -= 1;
                }
                pos += 1;
            }
            let kf_body = &css[kf_start..pos - 1];
            let kf_frames = parse_keyframes(kf_body);
            keyframes.push(crate::scene::KeyframeAnimation {
                name: anim_name,
                keyframes: kf_frames,
            });
            continue;
        }

        // Find declarations (everything before '}')
        let decl_start = pos;
        let mut brace_depth = 1;
        while pos < bytes.len() && brace_depth > 0 {
            if bytes[pos] == b'{' {
                brace_depth += 1;
            } else if bytes[pos] == b'}' {
                brace_depth -= 1;
            }
            pos += 1;
        }
        let decl_str = &css[decl_start..pos - 1];

        // Parse selector: each space-separated token is a segment (ancestor chain)
        // Each token can be compound: "div.btn.active" = one segment with multiple parts
        let selectors: Vec<Selector> = selector_str
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }
                let mut segments = Vec::new();
                let mut hover = false;

                for token in s.split_whitespace() {
                    let (base, is_hover) = if let Some(stripped) = token.strip_suffix(":hover") {
                        (stripped, true)
                    } else {
                        (token, false)
                    };
                    if is_hover {
                        hover = true;
                    }

                    // Parse one token into a segment (may have multiple parts)
                    let mut parts = Vec::new();
                    parse_selector_token(base, &mut parts);

                    if !parts.is_empty() {
                        segments.push(SelectorSegment { parts });
                    }
                }

                if segments.is_empty() {
                    return None;
                }
                Some(Selector { segments, hover })
            })
            .collect();

        // Parse declarations
        let declarations: Vec<(String, String)> = decl_str
            .split(';')
            .filter_map(|d| {
                let d = d.trim();
                if d.is_empty() {
                    return None;
                }
                let (prop, val) = d.split_once(':')?;
                Some((prop.trim().to_string(), val.trim().to_string()))
            })
            .collect();

        for selector in selectors {
            let specificity = selector.specificity();
            rules.push(CssRule {
                selector,
                declarations: declarations.clone(),
                specificity,
            });
        }
    }

    // Sort by specificity (lower first, so higher specificity overwrites)
    rules.sort_by_key(|r| r.specificity);
    (rules, keyframes)
}

fn parse_selector_token(token: &str, parts: &mut Vec<SelectorPart>) {
    if token == "*" {
        parts.push(SelectorPart::Universal);
        return;
    }
    if token.is_empty() {
        return;
    }

    let mut current = String::new();

    for c in token.chars() {
        if (c == '.' || c == '#') && !current.is_empty() {
            flush_selector_part(&current, parts);
            current = String::new();
        }
        current.push(c);
    }

    if !current.is_empty() {
        flush_selector_part(&current, parts);
    }
}

fn flush_selector_part(s: &str, parts: &mut Vec<SelectorPart>) {
    if let Some(id) = s.strip_prefix('#') {
        parts.push(SelectorPart::Id(id.to_string()));
    } else if let Some(class) = s.strip_prefix('.') {
        parts.push(SelectorPart::Class(class.to_string()));
    } else {
        parts.push(SelectorPart::Tag(s.to_string()));
    }
}

fn parse_keyframes(body: &str) -> Vec<crate::scene::Keyframe> {
    let mut frames = Vec::new();
    let mut pos = 0;
    let bytes = body.as_bytes();

    while pos < bytes.len() {
        while pos < bytes.len() && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        // Find percentage/from/to (before '{')
        let start = pos;
        while pos < bytes.len() && bytes[pos] != b'{' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        let pct_str = body[start..pos].trim();
        pos += 1;

        // Find declarations (before '}')
        let decl_start = pos;
        let mut depth = 1;
        while pos < bytes.len() && depth > 0 {
            if bytes[pos] == b'{' {
                depth += 1;
            } else if bytes[pos] == b'}' {
                depth -= 1;
            }
            pos += 1;
        }
        let decl_str = &body[decl_start..pos - 1];

        let percent = match pct_str {
            "from" => 0.0,
            "to" => 1.0,
            s => s
                .strip_suffix('%')
                .and_then(|p| p.trim().parse::<f32>().ok())
                .map(|p| p / 100.0)
                .unwrap_or(0.0),
        };

        let mut style = ResolvedStyle::default();
        parse_inline_style(decl_str, &mut style);

        frames.push(crate::scene::Keyframe { percent, style });
    }

    frames.sort_by(|a, b| a.percent.partial_cmp(&b.percent).unwrap());
    frames
}

pub fn apply_styles(scene: &mut SceneGraph, rules: &[CssRule]) {
    let node_count = scene.node_count();
    for i in 0..node_count {
        let node_id = NodeId(i);

        // Apply matching non-hover rules in specificity order
        for rule in rules {
            if rule.selector.is_hover() {
                continue;
            }
            let matches = if rule.selector.segments.len() > 1 {
                rule.selector
                    .matches_with_ancestors(scene.get(node_id), scene)
            } else {
                rule.selector.matches(scene.get(node_id))
            };
            if matches {
                for (prop, val) in &rule.declarations {
                    apply_property(&mut scene.nodes[i].style, prop, val);
                }
            }
        }

        // Apply inline styles (highest specificity)
        let inline_style = scene.get(node_id).attributes.get("style").cloned();
        if let Some(style_str) = inline_style {
            parse_inline_style(&style_str, &mut scene.nodes[i].style);
        }

        // Inherit color and font properties from parent
        if let Some(parent_id) = scene.get(node_id).parent {
            let parent_color = scene.get(parent_id).style.color;
            let parent_font = scene.get(parent_id).style.font_family.clone();

            let node_has_color = scene.nodes[i]
                .attributes
                .get("style")
                .map(|s| {
                    // Check for standalone "color:" not "background-color:"
                    s.split(';').any(|decl| {
                        let prop = decl.split(':').next().unwrap_or("").trim();
                        prop == "color"
                    })
                })
                .unwrap_or(false);
            let node_has_class_color = rules.iter().any(|r| {
                r.selector.matches(&scene.nodes[i])
                    && r.declarations.iter().any(|(p, _)| p == "color")
            });

            if !node_has_color && !node_has_class_color {
                scene.nodes[i].style.color = parent_color;
            }

            if scene.nodes[i].style.font_family.is_empty() {
                scene.nodes[i].style.font_family = parent_font;
            }
        }

        // Collect hover styles: start from the node's resolved style, apply hover rules on top
        let hover_rules: Vec<&CssRule> = rules
            .iter()
            .filter(|r| {
                r.selector.is_hover()
                    && if r.selector.segments.len() > 1 {
                        r.selector.matches_with_ancestors(&scene.nodes[i], scene)
                    } else {
                        r.selector.matches(&scene.nodes[i])
                    }
            })
            .collect();

        if !hover_rules.is_empty() {
            let mut hover = scene.nodes[i].style.clone();
            for rule in &hover_rules {
                for (prop, val) in &rule.declarations {
                    apply_property(&mut hover, prop, val);
                }
            }
            scene.nodes[i].hover_style = Some(hover);
            scene.nodes[i].base_style = Some(scene.nodes[i].style.clone());
        }

        // Store image src
        let img_src = scene.nodes[i].attributes.get("src").cloned();
        if scene.nodes[i].kind == crate::scene::ElementKind::Img {
            scene.nodes[i].image_src = img_src;
        }

        // Set up CSS animation if node has animation-name
        let anim_name = scene.nodes[i].style.animation_name.clone();
        if !anim_name.is_empty() {
            if let Some(kf_anim) = scene.keyframes.iter().find(|k| k.name == anim_name) {
                scene.nodes[i].animation = Some(crate::scene::AnimationState {
                    animation_name: anim_name,
                    duration: scene.nodes[i].style.animation_duration,
                    delay: scene.nodes[i].style.animation_delay,
                    iteration_count: scene.nodes[i].style.animation_iteration_count,
                    direction: scene.nodes[i].style.animation_direction,
                    start_time: None, // Started on first frame
                    current_iteration: 0.0,
                    keyframes: kf_anim.keyframes.clone(),
                });
            }
        }
    }
}
