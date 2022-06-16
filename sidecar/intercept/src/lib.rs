mod http_filter_context;
mod record;
mod sony_flake;

use http_filter_context::HttpFilterContext;
use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use record::Record;
use serde::{Deserialize, Serialize};
use sony_flake::SonyFlakeEntity;
use std::time::Duration;

const COLLECTOR_SERVICE_UPSTREAM: &str = "collector-service";
const OUTBOUND_RECORD: &str = "outbound_record";
const OUTBOUND_REPLAY: &str = "outbound_replay";
const INBOUND: &str = "inbound";
const BOOTSTRAP: &str = "bootstrap";
const SHARED_QUEUE_NAME: &str = "record_json";
const VM_ID: &str = "intercept";
const SHARED_DATA_NAME: &str = "trace_id";

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginContext {
            config: PluginConfig{
                plugin_type:"none".to_string()  ,
                host:None,
                record_path:None,
                replay_path:None,
            },
            sfe: SonyFlakeEntity::new_default(),
            queue_id: 0,
        })
    });
}}

#[derive(Serialize, Deserialize)]
struct PluginConfig {
    plugin_type: String,
    host: Option<String>,
    record_path: Option<String>,
    replay_path: Option<String>,
}
struct PluginContext {
    config: PluginConfig,
    sfe: SonyFlakeEntity,
    queue_id: u32,
}

impl Context for PluginContext {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        _body_size: usize,
        _num_trailers: usize,
    ) {
        info!(
            "[{}] http call response status:{}",
            self.config.plugin_type,
            self.get_http_call_response_header(":status").unwrap()
        )
    }
}

impl RootContext for PluginContext {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            self.config = serde_json::from_str(&*plugin_config).unwrap();
            info!("[{}] plugin started", self.config.plugin_type)
        }
        if self.config.plugin_type == BOOTSTRAP {
            let queue_id = self.register_shared_queue(SHARED_QUEUE_NAME);
            info!(
                "[{}] register shared queue:{}",
                self.config.plugin_type, queue_id
            )
        } else {
            if let Some(queue_id) = self.resolve_shared_queue(VM_ID, SHARED_QUEUE_NAME) {
                self.queue_id = queue_id
            }
            info!(
                "[{}] register shared queue:{}",
                self.config.plugin_type, self.queue_id
            )
        }
        true
    }
    fn on_queue_ready(&mut self, queue_id: u32) {
        if let Some(bytes) = self.dequeue_shared_queue(queue_id).expect("wrong queue") {
            info!("[{}] record", self.config.plugin_type);
            let host = self.config.host.as_ref().unwrap();
            let replay_path = self.config.record_path.as_ref().unwrap();
            self.dispatch_http_call(
                COLLECTOR_SERVICE_UPSTREAM,
                vec![
                    (":method", "POST"),
                    (":path", replay_path.as_str()),
                    (":authority", host.as_str()),
                ],
                Option::Some(&*bytes),
                vec![],
                Duration::from_secs(2),
            )
            .expect("dispatch http call error");
        }
    }
    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        info!(
            "[{}] http context started, context_id:{}",
            self.config.plugin_type, context_id
        );
        let mut trace_id: String = self.sfe.get_id(self.get_current_time()).to_string();
        if self.config.plugin_type == OUTBOUND_RECORD {
            if let (Some(bytes), _cas) = self.get_shared_data(SHARED_DATA_NAME) {
                trace_id = String::from_utf8(bytes).unwrap();
            }
        } else if self.config.plugin_type == INBOUND {
            match self.set_shared_data(
                SHARED_DATA_NAME,
                Some(trace_id.to_string().as_bytes()),
                None,
            ) {
                Ok(_) => {
                    info!(
                        "[{}] shared context id:{}",
                        self.config.plugin_type, trace_id
                    );
                }
                Err(cause) => panic!("unexpected status: {:?}", cause),
            }
        }
        let host;
        if self.config.host.is_some() {
            host = Some(self.config.host.as_ref().unwrap().to_string())
        } else {
            host = None
        }
        let replay_path;
        if self.config.replay_path.is_some() {
            replay_path = Some(self.config.replay_path.as_ref().unwrap().to_string())
        } else {
            replay_path = None
        }
        let record_path;
        if self.config.record_path.is_some() {
            record_path = Some(self.config.record_path.as_ref().unwrap().to_string())
        } else {
            record_path = None
        }
        Some(Box::new(HttpFilterContext {
            trace_id,
            queue_id: self.queue_id,
            config: PluginConfig {
                plugin_type: (&*self.config.plugin_type).to_string(),
                host,
                record_path,
                replay_path,
            },
            record: Record {
                plugin_type: None,
                trace_id: None,
                request_headers: None,
                request_body: None,
                response_headers: vec![],
                response_body: "".to_string(),
            },
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
