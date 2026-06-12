use boa_engine::property::Attribute;
use boa_engine::NativeFunction;
use boa_engine::{Context, JsArgs, JsValue, Source};
use std::cell::RefCell;
use std::rc::Rc;

use crate::scene::*;

#[derive(Debug, Clone)]
pub enum DomMutation {
    SetTextContent(String, String),
    SetStyle(String, String, String),
    AddClass(String, String),
    RemoveClass(String, String),
}

pub struct JsEngine {
    context: Context,
    mutations: Rc<RefCell<Vec<DomMutation>>>,
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JsEngine {
    pub fn new() -> Self {
        let context = Context::default();
        let mutations = Rc::new(RefCell::new(Vec::new()));

        let mut engine = Self { context, mutations };
        engine.register_dom_api();
        engine
    }

    fn register_dom_api(&mut self) {
        let mutations = self.mutations.clone();

        // SAFETY: from_closure requires the closure is valid for 'static.
        // The closure captures only Rc<RefCell<Vec<DomMutation>>> (no borrowed refs),
        // so it satisfies the 'static bound.
        let get_element = unsafe {
            NativeFunction::from_closure(move |_, args, ctx| {
                let id = args
                    .get_or_undefined(0)
                    .to_string(ctx)?
                    .to_std_string_escaped();

                let m = mutations.clone();
                let element_id = id.clone();

                let obj = boa_engine::JsObject::with_null_proto();

                // setTextContent(text)
                {
                    let m = m.clone();
                    let eid = element_id.clone();
                    let func = NativeFunction::from_closure(move |_, args, ctx| {
                        let text = args
                            .get_or_undefined(0)
                            .to_string(ctx)?
                            .to_std_string_escaped();
                        m.borrow_mut()
                            .push(DomMutation::SetTextContent(eid.clone(), text));
                        Ok(JsValue::undefined())
                    });
                    let js_func = func.to_js_function(ctx.realm());
                    obj.set(
                        boa_engine::js_string!("setTextContent"),
                        js_func,
                        false,
                        ctx,
                    )?;
                }

                // setStyle(property, value)
                {
                    let m = m.clone();
                    let eid = element_id.clone();
                    let func = NativeFunction::from_closure(move |_, args, ctx| {
                        let prop = args
                            .get_or_undefined(0)
                            .to_string(ctx)?
                            .to_std_string_escaped();
                        let val = args
                            .get_or_undefined(1)
                            .to_string(ctx)?
                            .to_std_string_escaped();
                        m.borrow_mut()
                            .push(DomMutation::SetStyle(eid.clone(), prop, val));
                        Ok(JsValue::undefined())
                    });
                    let js_func = func.to_js_function(ctx.realm());
                    obj.set(boa_engine::js_string!("setStyle"), js_func, false, ctx)?;
                }

                // addClass(className)
                {
                    let m = m.clone();
                    let eid = element_id.clone();
                    let func = NativeFunction::from_closure(move |_, args, ctx| {
                        let class = args
                            .get_or_undefined(0)
                            .to_string(ctx)?
                            .to_std_string_escaped();
                        m.borrow_mut()
                            .push(DomMutation::AddClass(eid.clone(), class));
                        Ok(JsValue::undefined())
                    });
                    let js_func = func.to_js_function(ctx.realm());
                    obj.set(boa_engine::js_string!("addClass"), js_func, false, ctx)?;
                }

                // removeClass(className)
                {
                    let m = m.clone();
                    let eid = element_id.clone();
                    let func = NativeFunction::from_closure(move |_, args, ctx| {
                        let class = args
                            .get_or_undefined(0)
                            .to_string(ctx)?
                            .to_std_string_escaped();
                        m.borrow_mut()
                            .push(DomMutation::RemoveClass(eid.clone(), class));
                        Ok(JsValue::undefined())
                    });
                    let js_func = func.to_js_function(ctx.realm());
                    obj.set(boa_engine::js_string!("removeClass"), js_func, false, ctx)?;
                }

                obj.set(
                    boa_engine::js_string!("id"),
                    JsValue::from(boa_engine::js_string!(element_id.as_str())),
                    false,
                    ctx,
                )?;

                Ok(JsValue::from(obj))
            })
        };

        let document = boa_engine::JsObject::with_null_proto();
        let get_elem_func = get_element.to_js_function(self.context.realm());
        document
            .set(
                boa_engine::js_string!("getElementById"),
                get_elem_func,
                false,
                &mut self.context,
            )
            .ok();

        self.context
            .register_global_property(
                boa_engine::js_string!("document"),
                JsValue::from(document),
                Attribute::all(),
            )
            .ok();

        // console.log
        let console = boa_engine::JsObject::with_null_proto();
        let log_fn = NativeFunction::from_fn_ptr(|_, args, ctx| {
            let msg = args
                .get_or_undefined(0)
                .to_string(ctx)?
                .to_std_string_escaped();
            println!("[foundry:js] {}", msg);
            Ok(JsValue::undefined())
        });
        let log_js_func = log_fn.to_js_function(self.context.realm());
        console
            .set(
                boa_engine::js_string!("log"),
                log_js_func,
                false,
                &mut self.context,
            )
            .ok();

        self.context
            .register_global_property(
                boa_engine::js_string!("console"),
                JsValue::from(console),
                Attribute::all(),
            )
            .ok();
    }

    pub fn execute(&mut self, code: &str) -> Result<(), String> {
        self.mutations.borrow_mut().clear();
        // boa_engine::Context::eval is the official API for JS execution
        self.context
            .eval(Source::from_bytes(code))
            .map_err(|e| format!("JS error: {}", e))?;
        Ok(())
    }

    pub fn take_mutations(&self) -> Vec<DomMutation> {
        self.mutations.borrow_mut().drain(..).collect()
    }

    pub fn apply_mutations(&self, scene: &mut SceneGraph, mutations: &[DomMutation]) {
        for mutation in mutations {
            match mutation {
                DomMutation::SetTextContent(element_id, text) => {
                    if let Some(node_id) = scene.find_by_element_id(element_id) {
                        let children: Vec<NodeId> = scene.get(node_id).children.clone();
                        let text_child = children
                            .iter()
                            .find(|&&c| scene.get(c).kind == ElementKind::Text);
                        if let Some(&child_id) = text_child {
                            scene.get_mut(child_id).text_content = Some(text.clone());
                            scene.get_mut(child_id).dirty = true;
                        } else {
                            let child = scene.add_node(ElementKind::Text, "text".to_string());
                            scene.get_mut(child).text_content = Some(text.clone());
                            scene.add_child(node_id, child);
                        }
                        scene.mark_dirty_recursive(node_id);
                    }
                }
                DomMutation::SetStyle(element_id, prop, val) => {
                    if let Some(node_id) = scene.find_by_element_id(element_id) {
                        crate::css::apply_property(&mut scene.get_mut(node_id).style, prop, val);
                        scene.mark_dirty_recursive(node_id);
                    }
                }
                DomMutation::AddClass(element_id, class) => {
                    if let Some(node_id) = scene.find_by_element_id(element_id) {
                        let node = scene.get_mut(node_id);
                        if !node.classes.contains(class) {
                            node.classes.push(class.clone());
                            node.dirty = true;
                        }
                    }
                }
                DomMutation::RemoveClass(element_id, class) => {
                    if let Some(node_id) = scene.find_by_element_id(element_id) {
                        let node = scene.get_mut(node_id);
                        node.classes.retain(|c| c != class);
                        node.dirty = true;
                    }
                }
            }
        }
    }
}
