// Parser and CSS tests for Foundry
// Tests the compile-time pipeline (no GPU needed)

use foundry_runtime::css::*;
use foundry_runtime::scene::*;

#[test]
fn test_parse_color_hex() {
    let c = parse_color("#ff0000").unwrap();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.a, 1.0);
}

#[test]
fn test_parse_color_hex_short() {
    let c = parse_color("#f00").unwrap();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
}

#[test]
fn test_parse_color_named() {
    let c = parse_color("white").unwrap();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 1.0);
    assert_eq!(c.b, 1.0);

    let c = parse_color("transparent").unwrap();
    assert_eq!(c.a, 0.0);
}

#[test]
fn test_parse_color_rgb() {
    let c = parse_color("rgb(255, 128, 0)").unwrap();
    assert_eq!(c.r, 1.0);
    assert!((c.g - 128.0 / 255.0).abs() < 0.01);
    assert_eq!(c.b, 0.0);
}

#[test]
fn test_parse_color_rgba() {
    let c = parse_color("rgba(255, 0, 0, 0.5)").unwrap();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.a, 0.5);
}

#[test]
fn test_parse_length_px() {
    assert_eq!(parse_length("16px"), Some(SizeValue::Px(16.0)));
    assert_eq!(parse_length("0"), Some(SizeValue::Px(0.0)));
    assert_eq!(parse_length("auto"), Some(SizeValue::Auto));
}

#[test]
fn test_parse_length_percent() {
    assert_eq!(parse_length("50%"), Some(SizeValue::Percent(50.0)));
    assert_eq!(parse_length("100%"), Some(SizeValue::Percent(100.0)));
}

#[test]
fn test_parse_length_viewport() {
    assert_eq!(parse_length("100vh"), Some(SizeValue::Vh(100.0)));
    assert_eq!(parse_length("50vw"), Some(SizeValue::Vw(50.0)));
}

#[test]
fn test_parse_length_em_rem() {
    assert_eq!(parse_length("1.5em"), Some(SizeValue::Em(1.5)));
    assert_eq!(parse_length("2rem"), Some(SizeValue::Rem(2.0)));
}

#[test]
fn test_apply_property_display() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "display", "flex");
    assert_eq!(style.display, Display::Flex);

    apply_property(&mut style, "display", "none");
    assert_eq!(style.display, Display::None);
}

#[test]
fn test_apply_property_flexbox() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "flex-direction", "column");
    assert_eq!(style.flex_direction, FlexDirection::Column);

    apply_property(&mut style, "justify-content", "center");
    assert_eq!(style.justify_content, JustifyContent::Center);

    apply_property(&mut style, "align-items", "center");
    assert_eq!(style.align_items, AlignItems::Center);
}

#[test]
fn test_apply_property_colors() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "background-color", "#e94560");
    assert!((style.background_color.r - 233.0 / 255.0).abs() < 0.01);

    apply_property(&mut style, "color", "white");
    assert_eq!(style.color.r, 1.0);
    assert_eq!(style.color.g, 1.0);
    assert_eq!(style.color.b, 1.0);
}

#[test]
fn test_apply_property_font() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "font-size", "24px");
    assert_eq!(style.font_size, 24.0);

    apply_property(&mut style, "font-weight", "bold");
    assert_eq!(style.font_weight, 700);

    apply_property(&mut style, "font-weight", "300");
    assert_eq!(style.font_weight, 300);
}

#[test]
fn test_apply_property_margin_shorthand() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "margin", "10px 20px");
    assert_eq!(style.margin, [10.0, 20.0, 10.0, 20.0]);

    apply_property(&mut style, "margin", "5px");
    assert_eq!(style.margin, [5.0, 5.0, 5.0, 5.0]);
}

#[test]
fn test_apply_property_border() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "border", "2px solid #333");
    assert_eq!(style.border_width, [2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn test_apply_property_border_radius() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "border-radius", "8px");
    assert_eq!(style.border_radius, [8.0, 8.0, 8.0, 8.0]);
}

#[test]
fn test_apply_property_position() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "position", "absolute");
    assert_eq!(style.position, Position::Absolute);
}

#[test]
fn test_apply_property_overflow() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "overflow", "scroll");
    assert_eq!(style.overflow, Overflow::Scroll);
}

#[test]
fn test_parse_inline_style() {
    let mut style = ResolvedStyle::default();
    parse_inline_style("color: red; font-size: 24px; display: flex", &mut style);
    assert_eq!(style.color.r, 1.0);
    assert_eq!(style.font_size, 24.0);
    assert_eq!(style.display, Display::Flex);
}

#[test]
fn test_parse_stylesheet_simple() {
    let css = r#"
        .card {
            background-color: #16213e;
            padding: 40px;
            border-radius: 12px;
        }
        h1 {
            font-size: 24px;
            color: red;
        }
        #count {
            font-weight: bold;
        }
    "#;
    let rules = parse_stylesheet(css);
    assert_eq!(rules.len(), 3);

    // Check specificity ordering (tag < class < id)
    assert_eq!(rules[0].specificity, 1); // tag
    assert_eq!(rules[1].specificity, 10); // class
    assert_eq!(rules[2].specificity, 100); // id
}

fn make_selector(part: SelectorPart, hover: bool) -> Selector {
    Selector {
        segments: vec![SelectorSegment { parts: vec![part] }],
        hover,
    }
}

#[test]
fn test_selector_matching() {
    let mut node = SceneNode::new(NodeId(0), ElementKind::Div, "div".to_string());
    node.classes = vec!["card".to_string()];
    node.element_id = Some("main".to_string());

    assert!(make_selector(SelectorPart::Tag("div".to_string()), false).matches(&node));
    assert!(make_selector(SelectorPart::Class("card".to_string()), false).matches(&node));
    assert!(make_selector(SelectorPart::Id("main".to_string()), false).matches(&node));
    assert!(!make_selector(SelectorPart::Tag("span".to_string()), false).matches(&node));
    assert!(!make_selector(SelectorPart::Class("other".to_string()), false).matches(&node));
    assert!(make_selector(SelectorPart::Universal, false).matches(&node));
}

#[test]
fn test_element_kind_from_tag() {
    assert_eq!(ElementKind::from_tag("div"), ElementKind::Div);
    assert_eq!(ElementKind::from_tag("h1"), ElementKind::H1);
    assert_eq!(ElementKind::from_tag("button"), ElementKind::Button);
    assert_eq!(ElementKind::from_tag("unknown"), ElementKind::Unknown);
}

#[test]
fn test_element_kind_inline() {
    assert!(ElementKind::Span.is_inline());
    assert!(ElementKind::A.is_inline());
    assert!(ElementKind::Text.is_inline());
    assert!(!ElementKind::Div.is_inline());
    assert!(!ElementKind::P.is_inline());
}

#[test]
fn test_scene_graph_operations() {
    let mut scene = SceneGraph::new();
    let root = scene.add_node(ElementKind::Div, "div".to_string());
    let child1 = scene.add_node(ElementKind::P, "p".to_string());
    let child2 = scene.add_node(ElementKind::Span, "span".to_string());

    scene.add_child(root, child1);
    scene.add_child(root, child2);

    assert_eq!(scene.node_count(), 3);
    assert_eq!(scene.get(root).children.len(), 2);
    assert_eq!(scene.get(child1).parent, Some(root));
}

#[test]
fn test_scene_graph_find_by_id() {
    let mut scene = SceneGraph::new();
    let node = scene.add_node(ElementKind::Div, "div".to_string());
    scene.get_mut(node).element_id = Some("test".to_string());

    assert_eq!(scene.find_by_element_id("test"), Some(node));
    assert_eq!(scene.find_by_element_id("missing"), None);
}

#[test]
fn test_heading_default_styles() {
    let h1 = SceneNode::new(NodeId(0), ElementKind::H1, "h1".to_string());
    assert_eq!(h1.style.font_size, 32.0);
    assert_eq!(h1.style.font_weight, 700);

    let h6 = SceneNode::new(NodeId(1), ElementKind::H6, "h6".to_string());
    assert_eq!(h6.style.font_size, 12.0);
}

#[test]
fn test_default_style_values() {
    let style = ResolvedStyle::default();
    assert_eq!(style.display, Display::Block);
    assert_eq!(style.font_size, 16.0);
    assert_eq!(style.opacity, 1.0);
    assert_eq!(style.color, Color::BLACK);
    assert_eq!(style.background_color, Color::TRANSPARENT);
}

// ===== v0.2 Feature Tests =====

#[test]
fn test_hover_selector_parsing() {
    let css = r#"
        .btn:hover {
            background-color: #c73650;
        }
        #main:hover {
            color: blue;
        }
        div:hover {
            opacity: 0.8;
        }
    "#;
    let rules = parse_stylesheet(css);
    assert_eq!(rules.len(), 3);
    assert!(rules.iter().all(|r| r.selector.is_hover()));
}

#[test]
fn test_hover_selector_matching() {
    let mut node = SceneNode::new(NodeId(0), ElementKind::Div, "div".to_string());
    node.classes = vec!["btn".to_string()];

    let hover_sel = make_selector(SelectorPart::Class("btn".to_string()), true);
    assert!(hover_sel.matches(&node));
    assert!(hover_sel.is_hover());

    let normal_sel = make_selector(SelectorPart::Class("btn".to_string()), false);
    assert!(normal_sel.matches(&node));
    assert!(!normal_sel.is_hover());
}

#[test]
fn test_compound_selector_parsing() {
    let css = r#"
        .card h1 {
            color: red;
        }
        div.btn {
            background-color: blue;
        }
        #main .text {
            font-size: 14px;
        }
    "#;
    let rules = parse_stylesheet(css);
    assert_eq!(rules.len(), 3);
    // ".card h1" = 2 segments: [Class("card")], [Tag("h1")]
    assert_eq!(rules[0].selector.segments.len(), 2);
    // "div.btn" = 1 segment with 2 parts: [Tag("div"), Class("btn")]
    assert_eq!(rules[1].selector.segments.len(), 1);
    assert_eq!(rules[1].selector.segments[0].parts.len(), 2);
    // "#main .text" = 2 segments: [Id("main")], [Class("text")]
    assert_eq!(rules[2].selector.segments.len(), 2);
}

#[test]
fn test_transition_css_parsing() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "transition", "all 0.3s");
    assert!((style.transition_duration - 0.3).abs() < 0.01);
    assert_eq!(style.transition_property, TransitionProperty::All);
}

#[test]
fn test_style_lerp() {
    let a = ResolvedStyle {
        background_color: Color::from_rgba(255, 0, 0, 1.0),
        ..Default::default()
    };
    let b = ResolvedStyle {
        background_color: Color::from_rgba(0, 0, 255, 1.0),
        ..Default::default()
    };
    let mid = a.lerp(&b, 0.5);
    assert!((mid.background_color.r - 0.5).abs() < 0.01);
    assert!((mid.background_color.b - 0.5).abs() < 0.01);

    let start = a.lerp(&b, 0.0);
    assert!((start.background_color.r - 1.0).abs() < 0.01);

    let end = a.lerp(&b, 1.0);
    assert!((end.background_color.b - 1.0).abs() < 0.01);
}

#[test]
fn test_hover_styles_applied_to_scene() {
    let mut scene = SceneGraph::new();
    let node = scene.add_node(ElementKind::Div, "div".to_string());
    scene.get_mut(node).classes = vec!["btn".to_string()];

    let css = r#"
        .btn {
            background-color: red;
        }
        .btn:hover {
            background-color: blue;
        }
    "#;
    let rules = parse_stylesheet(css);
    apply_styles(&mut scene, &rules);

    // Normal style: red
    assert_eq!(scene.get(node).style.background_color.r, 1.0);
    assert_eq!(scene.get(node).style.background_color.b, 0.0);

    // Hover style should exist and be blue
    assert!(scene.get(node).hover_style.is_some());
    let hover = scene.get(node).hover_style.as_ref().unwrap();
    assert_eq!(hover.background_color.r, 0.0);
    assert_eq!(hover.background_color.b, 1.0);

    // Base style preserved
    assert!(scene.get(node).base_style.is_some());
}

#[test]
fn test_apply_property_z_index() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "z-index", "10");
    assert_eq!(style.z_index, 10);

    apply_property(&mut style, "z-index", "-5");
    assert_eq!(style.z_index, -5);
}

#[test]
fn test_parse_color_hex_with_alpha() {
    let c = parse_color("#ff000080").unwrap();
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert!((c.a - 128.0 / 255.0).abs() < 0.01);
}

#[test]
fn test_box_shorthand_3_values() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "margin", "10px 20px 30px");
    assert_eq!(style.margin, [10.0, 20.0, 30.0, 20.0]);
}

#[test]
fn test_box_shorthand_4_values() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "padding", "1px 2px 3px 4px");
    assert_eq!(style.padding, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_apply_property_text_align() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "text-align", "center");
    assert_eq!(style.text_align, TextAlign::Center);

    apply_property(&mut style, "text-align", "right");
    assert_eq!(style.text_align, TextAlign::Right);
}

#[test]
fn test_scene_node_hover_fields() {
    let node = SceneNode::new(NodeId(0), ElementKind::Button, "button".to_string());
    assert!(node.hover_style.is_none());
    assert!(node.base_style.is_none());
    assert!(!node.is_hovered);
    assert!(node.image_src.is_none());
}

#[test]
fn test_img_element() {
    let node = SceneNode::new(NodeId(0), ElementKind::Img, "img".to_string());
    assert_eq!(node.kind, ElementKind::Img);
}

// ===== Keyframe & Animation Tests =====

#[test]
fn test_keyframe_parsing() {
    let css = r#"
        @keyframes fadeIn {
            from { opacity: 0 }
            to { opacity: 1 }
        }
        .element { color: red; }
    "#;
    let (rules, keyframes) = parse_stylesheet_with_keyframes(css);
    assert_eq!(rules.len(), 1); // .element rule
    assert_eq!(keyframes.len(), 1);
    assert_eq!(keyframes[0].name, "fadeIn");
    assert_eq!(keyframes[0].keyframes.len(), 2);
    assert_eq!(keyframes[0].keyframes[0].percent, 0.0);
    assert_eq!(keyframes[0].keyframes[1].percent, 1.0);
}

#[test]
fn test_keyframe_percentage_stops() {
    let css = r#"
        @keyframes pulse {
            0% { opacity: 1 }
            50% { opacity: 0.5 }
            100% { opacity: 1 }
        }
    "#;
    let (_, keyframes) = parse_stylesheet_with_keyframes(css);
    assert_eq!(keyframes.len(), 1);
    assert_eq!(keyframes[0].keyframes.len(), 3);
    assert!((keyframes[0].keyframes[1].percent - 0.5).abs() < 0.01);
}

#[test]
fn test_animation_property_parsing() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "animation", "fadeIn 0.5s");
    assert_eq!(style.animation_name, "fadeIn");
    assert!((style.animation_duration - 0.5).abs() < 0.01);
}

#[test]
fn test_animation_infinite() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "animation", "pulse 2s infinite");
    assert_eq!(style.animation_name, "pulse");
    assert!(style.animation_iteration_count.is_infinite());
}

#[test]
fn test_animation_alternate() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "animation", "slide 1s infinite alternate");
    assert_eq!(style.animation_direction, AnimationDirection::Alternate);
}

#[test]
fn test_animation_duration_ms() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "animation", "fade 300ms");
    assert!((style.animation_duration - 0.3).abs() < 0.01);
}

#[test]
fn test_lerp_border_width() {
    let a = ResolvedStyle {
        border_width: [0.0; 4],
        ..Default::default()
    };
    let b = ResolvedStyle {
        border_width: [4.0; 4],
        ..Default::default()
    };
    let mid = a.lerp(&b, 0.5);
    assert!((mid.border_width[0] - 2.0).abs() < 0.01);
}

#[test]
fn test_lerp_opacity() {
    let a = ResolvedStyle {
        opacity: 1.0,
        ..Default::default()
    };
    let b = ResolvedStyle {
        opacity: 0.0,
        ..Default::default()
    };
    let mid = a.lerp(&b, 0.5);
    assert!((mid.opacity - 0.5).abs() < 0.01);
}

#[test]
fn test_multiple_keyframe_animations() {
    let css = r#"
        @keyframes fadeIn {
            from { opacity: 0 }
            to { opacity: 1 }
        }
        @keyframes slideUp {
            from { opacity: 0 }
            to { opacity: 1 }
        }
    "#;
    let (_, keyframes) = parse_stylesheet_with_keyframes(css);
    assert_eq!(keyframes.len(), 2);
    assert_eq!(keyframes[0].name, "fadeIn");
    assert_eq!(keyframes[1].name, "slideUp");
}

// ===== Compound Selector Edge Cases =====

#[test]
fn test_compound_selector_two_classes() {
    // ".card.active" = one segment with two Class parts
    let css = ".card.active { color: red; }";
    let rules = parse_stylesheet(css);
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].selector.segments.len(), 1);
    assert_eq!(rules[0].selector.segments[0].parts.len(), 2);
}

#[test]
fn test_compound_selector_tag_class_matches() {
    let css = "div.btn { color: red; }";
    let rules = parse_stylesheet(css);

    let mut node = SceneNode::new(NodeId(0), ElementKind::Div, "div".to_string());
    node.classes = vec!["btn".to_string()];
    // Should match: same element is both div and .btn
    assert!(rules[0].selector.matches(&node));

    let mut wrong = SceneNode::new(NodeId(1), ElementKind::Span, "span".to_string());
    wrong.classes = vec!["btn".to_string()];
    // Should NOT match: span is not div
    assert!(!rules[0].selector.matches(&wrong));
}

#[test]
fn test_descendant_selector_with_scene() {
    let css = ".card h1 { color: red; }";
    let rules = parse_stylesheet(css);

    let mut scene = SceneGraph::new();
    let card = scene.add_node(ElementKind::Div, "div".to_string());
    scene.get_mut(card).classes = vec!["card".to_string()];
    let h1 = scene.add_node(ElementKind::H1, "h1".to_string());
    scene.add_child(card, h1);

    // h1 inside .card should match
    assert!(rules[0]
        .selector
        .matches_with_ancestors(scene.get(h1), &scene));

    // card itself should NOT match (it's not h1)
    assert!(!rules[0]
        .selector
        .matches_with_ancestors(scene.get(card), &scene));
}

#[test]
fn test_parse_time_function() {
    let mut style = ResolvedStyle::default();
    apply_property(&mut style, "transition-duration", "0.3s");
    assert!((style.transition_duration - 0.3).abs() < 0.01);

    apply_property(&mut style, "transition-duration", "300ms");
    assert!((style.transition_duration - 0.3).abs() < 0.01);
}

// ===== HTML Parser Tests =====

#[test]
fn test_parse_html_full() {
    use foundry_runtime::html;
    let (scene, styles, scripts, ext_styles, ext_scripts) = html::parse_html_full(
        r#"<html><head><style>body{color:red}</style><link rel="stylesheet" href="a.css"><script src="b.js"></script></head><body><div id="test">Hello</div><script>var x=1;</script></body></html>"#,
    );
    assert!(scene.node_count() > 0);
    assert_eq!(styles.len(), 1);
    assert_eq!(scripts.len(), 1);
    assert_eq!(ext_styles.len(), 1);
    assert_eq!(ext_styles[0], "a.css");
    assert_eq!(ext_scripts.len(), 1);
    assert_eq!(ext_scripts[0], "b.js");
}

#[test]
fn test_scene_graph_content_height() {
    let mut scene = SceneGraph::new();
    let root = scene.add_node(ElementKind::Div, "div".to_string());
    let child = scene.add_node(ElementKind::Div, "div".to_string());
    scene.add_child(root, child);
    // content_height defaults to 0, set by layout engine
    assert_eq!(scene.get(root).content_height, 0.0);
}
