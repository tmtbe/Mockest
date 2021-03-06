use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::format;

use crate::inbound_new_trace_proxy::new_inbound_new_trace_proxy;
use log::info;
use md5::Digest;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::inbound_record_proxy::new_inbound_record_proxy;
use crate::inbound_replay_proxy::new_inbound_replay_proxy;
use crate::inbound_serial_proxy::new_inbound_serial_proxy;
use crate::outbound_record_proxy::new_outbound_record_proxy;
use crate::outbound_replay_proxy::new_outbound_replay_proxy;

mod inbound_new_trace_proxy;
mod inbound_record_proxy;
mod inbound_replay_proxy;
mod inbound_serial_proxy;
mod outbound_record_proxy;
mod outbound_replay_proxy;
mod sony_flake;

const OUTBOUND_RECORD: &str = "outbound_record";
const OUTBOUND_REPLAY: &str = "outbound_replay";
const INBOUND_RECORD: &str = "inbound_record";
const INBOUND_REPLAY: &str = "inbound_replay";
const INBOUND_NEW_TRACE: &str = "inbound_new_trace";
const INBOUND_SERIAL: &str = "inbound_serial";
const SHARED_TRACE_ID_NAME: &str = "trace_id";
const R_INBOUND_TRACE_ID: &str = "r_inbound_trace_id";
const R_AUTHORITY: &str = "r_authority";
const R_INDEX: &str = "r_index";
const R_MATCH_TYPE: &str = "r_match_type";
const R_MATCH_INBOUND: &str = "r_match_inbound";
const R_MATCH_OUTBOUND: &str = "r_match_outbound";

lazy_static! {
    static ref RECORD_MAP: Mutex<HashMap<String, usize>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
    static ref REPLAY_MAP: Mutex<HashMap<String, usize>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}
#[derive(Serialize, Deserialize)]
struct Sign {
    request_headers: Vec<(String, String)>,
    request_body: Option<String>,
}

impl Sign {
    fn format(&mut self) {
        self.request_headers = self
            .request_headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .filter(|(k, v)| (!k.starts_with("x-")))
            .collect();
    }
}
fn clean_record_sign() {
    RECORD_MAP.lock().unwrap().clear()
}

fn add_sign_to_record(mut sign: Sign) -> usize {
    sign.format();
    let digest = md5::compute(serde_json::to_string(&sign).unwrap());
    let md5 = format!("{:x}", digest);
    if !RECORD_MAP.lock().unwrap().contains_key(md5.as_str()) {
        RECORD_MAP.lock().unwrap().insert(md5.clone(), 0);
    }
    let index = RECORD_MAP.lock().unwrap().get(md5.as_str()).unwrap() + 1;
    RECORD_MAP.lock().unwrap().insert(md5.clone(), index);
    info!(
        "[record] md5:{},size:{},index:{}",
        md5,
        RECORD_MAP.lock().unwrap().len(),
        index
    );
    return index;
}

fn clean_replay_sign() {
    REPLAY_MAP.lock().unwrap().clear()
}

fn add_sign_to_replay(mut sign: Sign) -> usize {
    sign.format();
    let digest = md5::compute(serde_json::to_string(&sign).unwrap());
    let md5 = format!("{:x}", digest);
    if !REPLAY_MAP.lock().unwrap().contains_key(md5.as_str()) {
        REPLAY_MAP.lock().unwrap().insert(md5.clone(), 0);
    }
    let index = REPLAY_MAP.lock().unwrap().get(md5.as_str()).unwrap() + 1;
    REPLAY_MAP.lock().unwrap().insert(md5.clone(), index);
    info!(
        "[replay] md5:{},size:{},index:{}",
        md5,
        RECORD_MAP.lock().unwrap().len(),
        index
    );
    return index;
}

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginContext {
            config: PluginConfig{
                plugin_type:"none".to_string(),
            },
            active_proxy: None,
        })
    });
}}

#[derive(Serialize, Deserialize)]
struct PluginConfig {
    plugin_type: String,
}
struct PluginContext {
    config: PluginConfig,
    active_proxy: Option<Box<dyn RootContext>>,
}

impl PluginContext {
    fn set_active_proxy(&mut self) {
        if self.config.plugin_type == INBOUND_RECORD {
            self.active_proxy = Some(new_inbound_record_proxy());
        } else if self.config.plugin_type == OUTBOUND_RECORD {
            self.active_proxy = Some(new_outbound_record_proxy());
        } else if self.config.plugin_type == INBOUND_REPLAY {
            self.active_proxy = Some(new_inbound_replay_proxy());
        } else if self.config.plugin_type == OUTBOUND_REPLAY {
            self.active_proxy = Some(new_outbound_replay_proxy());
        } else if self.config.plugin_type == INBOUND_NEW_TRACE {
            self.active_proxy = Some(new_inbound_new_trace_proxy());
        } else if self.config.plugin_type == INBOUND_SERIAL {
            self.active_proxy = Some(new_inbound_serial_proxy())
        }
    }
}

impl Context for PluginContext {}

impl RootContext for PluginContext {
    fn on_configure(&mut self, plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            self.config = serde_json::from_str(&*plugin_config).unwrap();
            info!("[{}] plugin started", self.config.plugin_type)
        }
        self.set_active_proxy();
        return self
            .active_proxy
            .as_mut()
            .unwrap()
            .on_configure(plugin_configuration_size);
    }
    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        self.active_proxy
            .as_ref()
            .unwrap()
            .create_http_context(context_id)
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
