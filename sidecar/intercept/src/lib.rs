mod inbound_record_proxy;
mod inbound_replay_proxy;
mod outbound_record_proxy;
mod outbound_replay_proxy;
mod sony_flake;

use crate::inbound_record_proxy::new_inbound_record_proxy;
use crate::inbound_replay_proxy::new_inbound_replay_proxy;
use crate::outbound_record_proxy::new_outbound_record_proxy;
use crate::outbound_replay_proxy::new_outbound_replay_proxy;
use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const COLLECTOR_SERVICE_UPSTREAM: &str = "collector";
const OUTBOUND_RECORD: &str = "outbound_record";
const OUTBOUND_REPLAY: &str = "outbound_replay";
const INBOUND_RECORD: &str = "inbound_record";
const INBOUND_REPLAY: &str = "inbound_replay";
const SHARED_QUEUE_NAME: &str = "record_json";
const VM_ID: &str = "intercept";
const SHARED_DATA_NAME: &str = "trace_id";

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