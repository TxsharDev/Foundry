use crate::scene::{ElementKind, NodeId, SceneGraph};
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

pub fn parse_html(html: &str) -> SceneGraph {
    let (scene, _, _, _, _) = parse_html_full(html);
    scene
}

/// Single-pass parse: returns (scene, inline_styles, inline_scripts, ext_style_hrefs, ext_script_srcs)
pub fn parse_html_full(
    html: &str,
) -> (
    SceneGraph,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
) {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .expect("failed to parse HTML");

    let mut scene = SceneGraph::new();
    let mut styles = Vec::new();
    let mut scripts = Vec::new();
    let mut ext_styles = Vec::new();
    let mut ext_scripts = Vec::new();

    walk_node_full(
        &dom.document,
        &mut scene,
        None,
        &mut styles,
        &mut scripts,
        &mut ext_styles,
        &mut ext_scripts,
    );

    (scene, styles, scripts, ext_styles, ext_scripts)
}

fn walk_node_full(
    handle: &Handle,
    scene: &mut SceneGraph,
    parent: Option<NodeId>,
    styles: &mut Vec<String>,
    scripts: &mut Vec<String>,
    ext_styles: &mut Vec<String>,
    ext_scripts: &mut Vec<String>,
) {
    match &handle.data {
        NodeData::Document => {
            for child in handle.children.borrow().iter() {
                walk_node_full(
                    child,
                    scene,
                    parent,
                    styles,
                    scripts,
                    ext_styles,
                    ext_scripts,
                );
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.to_string();

            // Extract <style> contents
            if tag == "style" {
                for child in handle.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        styles.push(contents.borrow().to_string());
                    }
                }
                return;
            }

            // Extract <script> contents or src
            if tag == "script" {
                let attrs_ref = attrs.borrow();
                let src = attrs_ref
                    .iter()
                    .find(|a| a.name.local.to_string() == "src")
                    .map(|a| a.value.to_string());
                if let Some(src_path) = src {
                    ext_scripts.push(src_path);
                } else {
                    for child in handle.children.borrow().iter() {
                        if let NodeData::Text { contents } = &child.data {
                            scripts.push(contents.borrow().to_string());
                        }
                    }
                }
                return;
            }

            // Extract <link rel="stylesheet" href="...">
            if tag == "link" {
                let attrs_ref = attrs.borrow();
                let is_stylesheet = attrs_ref.iter().any(|a| {
                    a.name.local.to_string() == "rel" && a.value.to_string() == "stylesheet"
                });
                if is_stylesheet {
                    if let Some(href) = attrs_ref
                        .iter()
                        .find(|a| a.name.local.to_string() == "href")
                    {
                        ext_styles.push(href.value.to_string());
                    }
                }
                return;
            }

            // Skip non-renderable head elements
            match tag.as_str() {
                "head" | "meta" | "title" => {
                    for child in handle.children.borrow().iter() {
                        walk_node_full(
                            child,
                            scene,
                            parent,
                            styles,
                            scripts,
                            ext_styles,
                            ext_scripts,
                        );
                    }
                    return;
                }
                _ => {}
            }

            // Build scene node
            let kind = ElementKind::from_tag(&tag);
            let id = scene.add_node(kind, tag.clone());

            let attrs_ref = attrs.borrow();
            for attr in attrs_ref.iter() {
                let attr_name = attr.name.local.to_string();
                let attr_value = attr.value.to_string();

                match attr_name.as_str() {
                    "id" => {
                        scene.get_mut(id).element_id = Some(attr_value.clone());
                    }
                    "class" => {
                        scene.get_mut(id).classes = attr_value
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    _ => {}
                }

                if let Some(event) = attr_name.strip_prefix("on") {
                    scene
                        .get_mut(id)
                        .event_handlers
                        .insert(event.to_string(), attr_value.clone());
                }

                scene.get_mut(id).attributes.insert(attr_name, attr_value);
            }

            if let Some(parent_id) = parent {
                scene.add_child(parent_id, id);
            }

            for child in handle.children.borrow().iter() {
                walk_node_full(
                    child,
                    scene,
                    Some(id),
                    styles,
                    scripts,
                    ext_styles,
                    ext_scripts,
                );
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return;
            }

            let id = scene.add_node(ElementKind::Text, "text".to_string());
            scene.get_mut(id).text_content = Some(trimmed.to_string());

            if let Some(parent_id) = parent {
                scene.add_child(parent_id, id);
            }
        }
        _ => {}
    }
}

// Legacy single-purpose walker, kept for parse_html() convenience wrapper
#[allow(dead_code)]
fn walk_node(handle: &Handle, scene: &mut SceneGraph, parent: Option<NodeId>) {
    let node = handle;

    match &node.data {
        NodeData::Document => {
            for child in node.children.borrow().iter() {
                walk_node(child, scene, parent);
            }
        }
        NodeData::Element { name, attrs, .. } => {
            let tag = name.local.to_string();
            let kind = ElementKind::from_tag(&tag);

            // Skip head, meta, title, script (handled separately), link, style (handled separately)
            match tag.as_str() {
                "head" | "meta" | "title" | "link" => {
                    // Still walk children to find script/style tags
                    for child in node.children.borrow().iter() {
                        if let NodeData::Element { name, .. } = &child.data {
                            let child_tag = name.local.to_string();
                            if child_tag == "style" || child_tag == "script" {
                                walk_node(child, scene, parent);
                            }
                        }
                    }
                    return;
                }
                _ => {}
            }

            let id = scene.add_node(kind, tag.clone());

            // Process attributes
            let attrs = attrs.borrow();
            for attr in attrs.iter() {
                let attr_name = attr.name.local.to_string();
                let attr_value = attr.value.to_string();

                match attr_name.as_str() {
                    "id" => {
                        scene.get_mut(id).element_id = Some(attr_value.clone());
                    }
                    "class" => {
                        scene.get_mut(id).classes = attr_value
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect();
                    }
                    "style" => {
                        // Inline styles are parsed later by the CSS module
                    }
                    _ => {}
                }

                // Store event handlers
                if let Some(event) = attr_name.strip_prefix("on") {
                    scene
                        .get_mut(id)
                        .event_handlers
                        .insert(event.to_string(), attr_value.clone());
                }

                scene.get_mut(id).attributes.insert(attr_name, attr_value);
            }

            if let Some(parent_id) = parent {
                scene.add_child(parent_id, id);
            }

            // Walk children
            for child in node.children.borrow().iter() {
                walk_node(child, scene, Some(id));
            }
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return;
            }

            let id = scene.add_node(ElementKind::Text, "text".to_string());
            scene.get_mut(id).text_content = Some(trimmed.to_string());

            if let Some(parent_id) = parent {
                scene.add_child(parent_id, id);
            }
        }
        _ => {
            // Comments, processing instructions, etc. - skip
        }
    }
}
