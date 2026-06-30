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

/// In-memory localStorage backed by a JSON file for persistence.
#[derive(Debug, Clone, Default)]
pub struct LocalStorage {
    data: std::collections::HashMap<String, String>,
    file_path: Option<std::path::PathBuf>,
}

impl LocalStorage {
    pub fn new(file_path: Option<std::path::PathBuf>) -> Self {
        let mut storage = Self {
            data: std::collections::HashMap::new(),
            file_path: file_path.clone(),
        };
        if let Some(path) = &file_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, String>>(
                    &content,
                ) {
                    storage.data = map;
                }
            }
        }
        storage
    }

    pub fn get_item(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set_item(&mut self, key: String, value: String) {
        self.data.insert(key, value);
        self.persist();
    }

    pub fn remove_item(&mut self, key: &str) {
        self.data.remove(key);
        self.persist();
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.persist();
    }

    pub fn length(&self) -> usize {
        self.data.len()
    }

    fn persist(&self) {
        if let Some(path) = &self.file_path {
            if let Ok(json) = serde_json::to_string_pretty(&self.data) {
                // Atomic write: write to temp file, then rename.
                // Prevents data loss on crash or disk-full mid-write.
                let tmp = path.with_extension("json.tmp");
                if std::fs::write(&tmp, &json).is_ok() {
                    let _ = std::fs::rename(&tmp, path);
                }
            }
        }
    }
}

pub struct JsEngine {
    context: Context,
    mutations: Rc<RefCell<Vec<DomMutation>>>,
    storage: Rc<RefCell<LocalStorage>>,
}

impl Default for JsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl JsEngine {
    pub fn new() -> Self {
        Self::with_storage_path(None)
    }

    pub fn with_storage_path(storage_path: Option<std::path::PathBuf>) -> Self {
        let context = Context::default();
        let mutations = Rc::new(RefCell::new(Vec::new()));
        let storage = Rc::new(RefCell::new(LocalStorage::new(storage_path)));

        let mut engine = Self {
            context,
            mutations,
            storage,
        };
        engine.register_dom_api();
        engine.register_storage_api();
        engine.register_fetch_api();
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

    fn register_storage_api(&mut self) {
        let storage = self.storage.clone();
        let ls = boa_engine::JsObject::with_null_proto();

        // getItem(key) -> string | null
        {
            let s = storage.clone();
            // SAFETY: captures Rc<RefCell<LocalStorage>> only; Rc<T>: 'static when T: 'static.
            let func = unsafe {
                NativeFunction::from_closure(move |_, args, ctx| {
                    let key = args
                        .get_or_undefined(0)
                        .to_string(ctx)?
                        .to_std_string_escaped();
                    let val = s.borrow().get_item(&key).cloned();
                    match val {
                        Some(v) => Ok(JsValue::from(boa_engine::js_string!(v.as_str()))),
                        None => Ok(JsValue::null()),
                    }
                })
            };
            let js_func = func.to_js_function(self.context.realm());
            ls.set(boa_engine::js_string!("getItem"), js_func, false, &mut self.context).ok();
        }

        // setItem(key, value)
        {
            let s = storage.clone();
            // SAFETY: captures Rc<RefCell<LocalStorage>> only; Rc<T>: 'static when T: 'static.
            let func = unsafe {
                NativeFunction::from_closure(move |_, args, ctx| {
                    let key = args
                        .get_or_undefined(0)
                        .to_string(ctx)?
                        .to_std_string_escaped();
                    let val = args
                        .get_or_undefined(1)
                        .to_string(ctx)?
                        .to_std_string_escaped();
                    s.borrow_mut().set_item(key, val);
                    Ok(JsValue::undefined())
                })
            };
            let js_func = func.to_js_function(self.context.realm());
            ls.set(boa_engine::js_string!("setItem"), js_func, false, &mut self.context).ok();
        }

        // removeItem(key)
        {
            let s = storage.clone();
            // SAFETY: captures Rc<RefCell<LocalStorage>> only; Rc<T>: 'static when T: 'static.
            let func = unsafe {
                NativeFunction::from_closure(move |_, args, ctx| {
                    let key = args
                        .get_or_undefined(0)
                        .to_string(ctx)?
                        .to_std_string_escaped();
                    s.borrow_mut().remove_item(&key);
                    Ok(JsValue::undefined())
                })
            };
            let js_func = func.to_js_function(self.context.realm());
            ls.set(boa_engine::js_string!("removeItem"), js_func, false, &mut self.context).ok();
        }

        // clear()
        {
            let s = storage.clone();
            // SAFETY: captures Rc<RefCell<LocalStorage>> only; Rc<T>: 'static when T: 'static.
            let func = unsafe {
                NativeFunction::from_closure(move |_, _args, _ctx| {
                    s.borrow_mut().clear();
                    Ok(JsValue::undefined())
                })
            };
            let js_func = func.to_js_function(self.context.realm());
            ls.set(boa_engine::js_string!("clear"), js_func, false, &mut self.context).ok();
        }

        self.context
            .register_global_property(
                boa_engine::js_string!("localStorage"),
                JsValue::from(ls),
                Attribute::all(),
            )
            .ok();
    }

    fn register_fetch_api(&mut self) {
        // Synchronous fetch(url) -> { ok: bool, status: number, text: string }
        // Blocking with 5s connect / 10s read timeout to avoid freezing the UI.
        // Body capped at 4 MB to prevent unbounded allocation from remote.
        let func = NativeFunction::from_fn_ptr(|_, args, ctx| {
            let url = args
                .get_or_undefined(0)
                .to_string(ctx)?
                .to_std_string_escaped();

            let result = boa_engine::JsObject::with_null_proto();

            let agent = ureq::Agent::new_with_config(
                ureq::config::Config::builder()
                    .timeout_connect(Some(std::time::Duration::from_secs(5)))
                    .timeout_recv_body(Some(std::time::Duration::from_secs(10)))
                    .build()
            );

            match agent.get(&url).call() {
                Ok(response) => {
                    let status = response.status().as_u16();
                    // Cap body at 4 MB to prevent unbounded allocation
                    let body = response
                        .into_body()
                        .with_config()
                        .limit(4 * 1024 * 1024)
                        .read_to_string()
                        .unwrap_or_default();
                    result
                        .set(
                            boa_engine::js_string!("ok"),
                            JsValue::from(status >= 200 && status < 300),
                            false,
                            ctx,
                        )
                        .ok();
                    result
                        .set(
                            boa_engine::js_string!("status"),
                            JsValue::from(status as i32),
                            false,
                            ctx,
                        )
                        .ok();
                    result
                        .set(
                            boa_engine::js_string!("text"),
                            JsValue::from(boa_engine::js_string!(body.as_str())),
                            false,
                            ctx,
                        )
                        .ok();
                }
                Err(e) => {
                    result
                        .set(
                            boa_engine::js_string!("ok"),
                            JsValue::from(false),
                            false,
                            ctx,
                        )
                        .ok();
                    result
                        .set(
                            boa_engine::js_string!("status"),
                            JsValue::from(0),
                            false,
                            ctx,
                        )
                        .ok();
                    let err_msg = e.to_string();
                    result
                        .set(
                            boa_engine::js_string!("text"),
                            JsValue::from(boa_engine::js_string!(err_msg.as_str())),
                            false,
                            ctx,
                        )
                        .ok();
                }
            }

            Ok(JsValue::from(result))
        });

        let js_func = func.to_js_function(self.context.realm());
        self.context
            .register_global_property(
                boa_engine::js_string!("fetch"),
                JsValue::from(js_func),
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
