use crate::scene::*;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Click(f32, f32),
    MouseMove(f32, f32),
    KeyPress(String),
    Scroll(f32, f32, f32, f32), // x, y, dx, dy
}

#[derive(Debug, Clone)]
pub struct EventResult {
    pub target: NodeId,
    pub event_type: String,
    pub handler_code: String,
}

pub struct EventSystem {
    pub hover_node: Option<NodeId>,
    pub focus_node: Option<NodeId>,
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            hover_node: None,
            focus_node: None,
        }
    }

    pub fn handle(&mut self, event: &AppEvent, scene: &mut SceneGraph) -> Vec<EventResult> {
        match event {
            AppEvent::Click(x, y) => self.handle_click(*x, *y, scene),
            AppEvent::MouseMove(x, y) => self.handle_mouse_move(*x, *y, scene),
            AppEvent::Scroll(x, y, dx, dy) => {
                self.handle_scroll(*x, *y, *dx, *dy, scene);
                Vec::new()
            }
            AppEvent::KeyPress(key) => self.handle_key(key, scene),
        }
    }

    fn handle_click(&mut self, x: f32, y: f32, scene: &SceneGraph) -> Vec<EventResult> {
        let mut results = Vec::new();

        if let Some(hit) = self.hit_test(x, y, scene) {
            // Walk up from hit node, collecting click handlers (bubble)
            let mut current = Some(hit);
            while let Some(node_id) = current {
                let node = scene.get(node_id);
                if let Some(handler) = node.event_handlers.get("click") {
                    results.push(EventResult {
                        target: hit,
                        event_type: "click".to_string(),
                        handler_code: handler.clone(),
                    });
                }
                current = node.parent;
            }
        }

        results
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32, scene: &mut SceneGraph) -> Vec<EventResult> {
        let mut results = Vec::new();
        let new_hover = self.hit_test(x, y, scene);

        if new_hover != self.hover_node {
            // Remove hover style from old node
            if let Some(old) = self.hover_node {
                let node = scene.get(old);
                let handler = node.event_handlers.get("mouseleave").cloned();
                let has_base = node.base_style.is_some();
                let has_transition = node.style.transition_duration > 0.0;
                if let Some(handler) = handler {
                    results.push(EventResult {
                        target: old,
                        event_type: "mouseleave".to_string(),
                        handler_code: handler,
                    });
                }
                if has_base {
                    let base = scene.get(old).base_style.clone().unwrap();
                    if has_transition {
                        // Start transition back to base
                        let current = scene.get(old).style.clone();
                        let node = scene.get_mut(old);
                        node.transition_from = Some(current);
                        node.transition_to = Some(base);
                        node.transition_start = Some(std::time::Instant::now());
                    } else {
                        scene.get_mut(old).style = base;
                    }
                    scene.get_mut(old).is_hovered = false;
                    scene.get_mut(old).dirty = true;
                }
            }
            // Apply hover style to new node
            if let Some(new) = new_hover {
                let node = scene.get(new);
                let handler = node.event_handlers.get("mouseenter").cloned();
                let has_hover = node.hover_style.is_some();
                let has_transition = node.style.transition_duration > 0.0
                    || node
                        .hover_style
                        .as_ref()
                        .map(|h| h.transition_duration > 0.0)
                        .unwrap_or(false);
                if let Some(handler) = handler {
                    results.push(EventResult {
                        target: new,
                        event_type: "mouseenter".to_string(),
                        handler_code: handler,
                    });
                }
                if has_hover {
                    let hover = scene.get(new).hover_style.clone().unwrap();
                    if has_transition {
                        // Start transition to hover style
                        let current = scene.get(new).style.clone();
                        let node = scene.get_mut(new);
                        node.transition_from = Some(current);
                        node.transition_to = Some(hover);
                        node.transition_start = Some(std::time::Instant::now());
                    } else {
                        scene.get_mut(new).style = hover;
                    }
                    scene.get_mut(new).is_hovered = true;
                    scene.get_mut(new).dirty = true;
                }
            }
            self.hover_node = new_hover;
        }

        results
    }

    fn handle_scroll(&mut self, x: f32, y: f32, _dx: f32, dy: f32, scene: &mut SceneGraph) {
        // Find the deepest scrollable container at (x, y)
        if let Some(root) = scene.root {
            if let Some(scroll_target) = self.find_scrollable(x, y, scene, root) {
                let node = scene.get_mut(scroll_target);
                let max_scroll = (node.content_height - node.layout.height).max(0.0);
                node.scroll_offset.1 = (node.scroll_offset.1 + dy * 30.0).clamp(0.0, max_scroll);
            }
        }
    }

    fn handle_key(&self, _key: &str, scene: &SceneGraph) -> Vec<EventResult> {
        let mut results = Vec::new();

        if let Some(focus) = self.focus_node {
            let node = scene.get(focus);
            if let Some(handler) = node.event_handlers.get("keypress") {
                results.push(EventResult {
                    target: focus,
                    event_type: "keypress".to_string(),
                    handler_code: handler.clone(),
                });
            }
        }

        results
    }

    fn hit_test(&self, x: f32, y: f32, scene: &SceneGraph) -> Option<NodeId> {
        if let Some(root) = scene.root {
            self.hit_test_recursive(x, y, scene, root)
        } else {
            None
        }
    }

    fn hit_test_recursive(
        &self,
        x: f32,
        y: f32,
        scene: &SceneGraph,
        node_id: NodeId,
    ) -> Option<NodeId> {
        let node = scene.get(node_id);

        if node.style.display == Display::None {
            return None;
        }

        let layout = &node.layout;
        // Account for scroll offset
        let scroll_y = node.scroll_offset.1;
        let in_bounds = x >= layout.x
            && x <= layout.x + layout.width
            && y >= layout.y
            && y <= layout.y + layout.height;

        if !in_bounds && node.style.overflow != Overflow::Visible {
            return None;
        }

        // Adjust child hit coords for this node's scroll offset
        let child_y = y + scroll_y;

        // Check children in reverse order (last drawn = on top)
        let children: Vec<NodeId> = node.children.clone();
        for &child_id in children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(x, child_y, scene, child_id) {
                return Some(hit);
            }
        }

        if in_bounds && !node.event_handlers.is_empty() {
            return Some(node_id);
        }

        if in_bounds {
            return Some(node_id);
        }

        None
    }

    fn find_scrollable(
        &self,
        x: f32,
        y: f32,
        scene: &SceneGraph,
        node_id: NodeId,
    ) -> Option<NodeId> {
        let node = scene.get(node_id);
        let layout = &node.layout;

        let in_bounds = x >= layout.x
            && x <= layout.x + layout.width
            && y >= layout.y
            && y <= layout.y + layout.height;

        if !in_bounds {
            return None;
        }

        // Check children first (deepest scrollable wins)
        let children: Vec<NodeId> = node.children.clone();
        for &child_id in children.iter().rev() {
            if let Some(found) = self.find_scrollable(x, y, scene, child_id) {
                return Some(found);
            }
        }

        if node.style.overflow == Overflow::Scroll {
            return Some(node_id);
        }

        None
    }
}
