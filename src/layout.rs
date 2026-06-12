use std::collections::HashMap;
use taffy::prelude::*;

use crate::scene::{
    AlignItems, Display, ElementKind, FlexDirection, FlexWrap, JustifyContent, LayoutRect, NodeId,
    Overflow, Position, ResolvedStyle, SceneGraph, SceneNode, SizeValue,
};

pub struct LayoutEngine {
    tree: TaffyTree,
    node_map: HashMap<NodeId, taffy::NodeId>,
    reverse_map: HashMap<taffy::NodeId, NodeId>,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            node_map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    pub fn compute(&mut self, scene: &mut SceneGraph, viewport_width: f32, viewport_height: f32) {
        self.tree = TaffyTree::new();
        self.node_map.clear();
        self.reverse_map.clear();

        if let Some(root_id) = scene.root {
            // Force html and body nodes to fill viewport (like browsers do)
            for i in 0..scene.nodes.len() {
                let tag = scene.nodes[i].tag.clone();
                if tag == "html" || tag == "body" {
                    if scene.nodes[i].style.width == SizeValue::Auto {
                        scene.nodes[i].style.width = SizeValue::Px(viewport_width);
                    }
                    if scene.nodes[i].style.height == SizeValue::Auto {
                        scene.nodes[i].style.min_height = SizeValue::Px(viewport_height);
                    }
                    // Body scrolls by default when content overflows
                    if tag == "body" && scene.nodes[i].style.overflow == Overflow::Visible {
                        scene.nodes[i].style.overflow = Overflow::Scroll;
                    }
                }
            }

            let taffy_root = self.build_taffy_tree(scene, root_id, viewport_width, viewport_height);

            self.tree
                .compute_layout(
                    taffy_root,
                    Size {
                        width: AvailableSpace::Definite(viewport_width),
                        height: AvailableSpace::Definite(viewport_height),
                    },
                )
                .ok();

            self.apply_layout(scene, root_id, 0.0, 0.0);
        }
    }

    fn build_taffy_tree(
        &mut self,
        scene: &SceneGraph,
        node_id: NodeId,
        vw: f32,
        vh: f32,
    ) -> taffy::NodeId {
        let node = scene.get(node_id);
        let style = &node.style;

        let taffy_style = self.convert_style(style, node, vw, vh, scene);

        let children: Vec<taffy::NodeId> = node
            .children
            .iter()
            .map(|&child_id| self.build_taffy_tree(scene, child_id, vw, vh))
            .collect();

        let taffy_node = self.tree.new_with_children(taffy_style, &children).unwrap();
        self.node_map.insert(node_id, taffy_node);
        self.reverse_map.insert(taffy_node, node_id);
        taffy_node
    }

    fn convert_style(
        &self,
        style: &ResolvedStyle,
        node: &SceneNode,
        vw: f32,
        vh: f32,
        scene_graph: &SceneGraph,
    ) -> Style {
        self.build_taffy_style(style, node, vw, vh, scene_graph)
    }

    #[allow(clippy::field_reassign_with_default)]
    fn build_taffy_style(
        &self,
        style: &ResolvedStyle,
        node: &SceneNode,
        vw: f32,
        vh: f32,
        scene_graph: &SceneGraph,
    ) -> Style {
        let mut ts = Style::default();

        // Display
        ts.display = match style.display {
            Display::Flex => taffy::Display::Flex,
            Display::Block => taffy::Display::Block,
            Display::None => taffy::Display::None,
            Display::Inline => taffy::Display::Flex, // approximate inline as flex
        };

        // Flex properties
        ts.flex_direction = match style.flex_direction {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        };

        ts.justify_content = Some(match style.justify_content {
            JustifyContent::Start => taffy::JustifyContent::FlexStart,
            JustifyContent::Center => taffy::JustifyContent::Center,
            JustifyContent::End => taffy::JustifyContent::FlexEnd,
            JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
        });

        ts.align_items = Some(match style.align_items {
            AlignItems::Stretch => taffy::AlignItems::Stretch,
            AlignItems::Start => taffy::AlignItems::FlexStart,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::End => taffy::AlignItems::FlexEnd,
        });

        ts.flex_wrap = match style.flex_wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
        };

        ts.flex_grow = style.flex_grow;
        ts.flex_shrink = style.flex_shrink;
        ts.gap = Size {
            width: LengthPercentage::Length(style.gap),
            height: LengthPercentage::Length(style.gap),
        };

        // Position
        ts.position = match style.position {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute | Position::Fixed => taffy::Position::Absolute,
        };

        ts.inset = Rect {
            top: self.convert_length_auto(style.top, vw, vh),
            right: self.convert_length_auto(style.right, vw, vh),
            bottom: self.convert_length_auto(style.bottom, vw, vh),
            left: self.convert_length_auto(style.left, vw, vh),
        };

        // Size
        ts.size = Size {
            width: self.convert_dimension(style.font_size, style.width, vw, vh),
            height: self.convert_dimension(style.font_size, style.height, vw, vh),
        };
        ts.min_size = Size {
            width: self.convert_dimension(style.font_size, style.min_width, vw, vh),
            height: self.convert_dimension(style.font_size, style.min_height, vw, vh),
        };
        ts.max_size = Size {
            width: self.convert_dimension(style.font_size, style.max_width, vw, vh),
            height: self.convert_dimension(style.font_size, style.max_height, vw, vh),
        };

        // Margin
        ts.margin = Rect {
            top: LengthPercentageAuto::Length(style.margin[0]),
            right: LengthPercentageAuto::Length(style.margin[1]),
            bottom: LengthPercentageAuto::Length(style.margin[2]),
            left: LengthPercentageAuto::Length(style.margin[3]),
        };

        // Padding
        ts.padding = Rect {
            top: LengthPercentage::Length(style.padding[0]),
            right: LengthPercentage::Length(style.padding[1]),
            bottom: LengthPercentage::Length(style.padding[2]),
            left: LengthPercentage::Length(style.padding[3]),
        };

        // Border
        ts.border = Rect {
            top: LengthPercentage::Length(style.border_width[0]),
            right: LengthPercentage::Length(style.border_width[1]),
            bottom: LengthPercentage::Length(style.border_width[2]),
            left: LengthPercentage::Length(style.border_width[3]),
        };

        // Text nodes: inherit font properties from parent for sizing
        if node.kind == ElementKind::Text {
            if let Some(text) = &node.text_content {
                // Use parent's font size since CSS is applied to the container, not the text node
                let font_size = if let Some(parent_id) = node.parent {
                    scene_graph.get(parent_id).style.font_size
                } else {
                    style.font_size
                };
                let font_weight = if let Some(parent_id) = node.parent {
                    scene_graph.get(parent_id).style.font_weight
                } else {
                    style.font_weight
                };
                let line_height = if let Some(parent_id) = node.parent {
                    scene_graph.get(parent_id).style.line_height
                } else {
                    style.line_height
                };

                let char_count = text.chars().count() as f32;
                let ratio = if font_weight >= 700 { 0.72 } else { 0.62 };
                let text_width = char_count * font_size * ratio;
                let text_height = font_size * line_height;
                ts.size = Size {
                    width: Dimension::Length(text_width),
                    height: Dimension::Length(text_height),
                };
            }
        }

        // Block display: default to column layout for children
        if style.display == Display::Block {
            ts.display = taffy::Display::Flex;
            ts.flex_direction = taffy::FlexDirection::Column;
        }

        ts
    }

    fn convert_dimension(&self, font_size: f32, val: SizeValue, vw: f32, vh: f32) -> Dimension {
        match val {
            SizeValue::Px(v) => Dimension::Length(v),
            SizeValue::Percent(v) => Dimension::Percent(v / 100.0),
            SizeValue::Em(v) => Dimension::Length(v * font_size),
            SizeValue::Rem(v) => Dimension::Length(v * 16.0),
            SizeValue::Vh(v) => Dimension::Length(v / 100.0 * vh),
            SizeValue::Vw(v) => Dimension::Length(v / 100.0 * vw),
            SizeValue::Auto => Dimension::Auto,
        }
    }

    fn convert_length_auto(&self, val: SizeValue, vw: f32, vh: f32) -> LengthPercentageAuto {
        match val {
            SizeValue::Px(v) => LengthPercentageAuto::Length(v),
            SizeValue::Percent(v) => LengthPercentageAuto::Percent(v / 100.0),
            SizeValue::Vh(v) => LengthPercentageAuto::Length(v / 100.0 * vh),
            SizeValue::Vw(v) => LengthPercentageAuto::Length(v / 100.0 * vw),
            _ => LengthPercentageAuto::Auto,
        }
    }

    fn apply_layout(&self, scene: &mut SceneGraph, node_id: NodeId, parent_x: f32, parent_y: f32) {
        if let Some(&taffy_node) = self.node_map.get(&node_id) {
            if let Ok(layout) = self.tree.layout(taffy_node) {
                let x = parent_x + layout.location.x;
                let y = parent_y + layout.location.y;

                let scene_node = scene.get_mut(node_id);
                scene_node.layout = LayoutRect {
                    x,
                    y,
                    width: layout.size.width,
                    height: layout.size.height,
                };
                scene_node.dirty = false;

                let children: Vec<NodeId> = scene_node.children.clone();
                for child_id in children {
                    self.apply_layout(scene, child_id, x, y);
                }

                // Compute content_height from children (for scroll)
                let mut max_bottom: f32 = 0.0;
                for &child_id in &scene.get(node_id).children {
                    let child = scene.get(child_id);
                    let child_bottom = child.layout.y + child.layout.height - y;
                    if child_bottom > max_bottom {
                        max_bottom = child_bottom;
                    }
                }
                scene.get_mut(node_id).content_height = max_bottom;
            }
        }
    }
}
