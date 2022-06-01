mod sony_flake;

use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use sony_flake::SonyFlakeEntity;
use std::time::Duration;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginContext {
            config: PluginConfig{
                plugin_type:"none".to_string()  ,
                host:None,
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
}
struct PluginContext {
    config: PluginConfig,
    sfe: SonyFlakeEntity,
    queue_id: u32,
}

impl Context for PluginContext {}

impl RootContext for PluginContext {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            self.config = serde_json::from_str(&*plugin_config).unwrap();
            info!("[{}] plugin started", self.config.plugin_type)
        }
        if self.config.plugin_type == "bootstrap" {
            let queue_id = self.register_shared_queue("record_json");
            info!(
                "[{}] register shared queue:{}",
                self.config.plugin_type, queue_id
            )
        } else {
            if let Some(queue_id) = self.resolve_shared_queue("record", "record_json") {
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
            let record = String::from_utf8(bytes).unwrap();
            info!("[{}] record {}", self.config.plugin_type, record);
            let host = self.config.host.as_ref().unwrap();
            self.dispatch_http_call(
                "record-service",
                vec![
                    (":method", "POST"),
                    (":path", "/post"),
                    (":authority", host.as_str()),
                ],
                Option::Some(record.as_ref()),
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
        if self.config.plugin_type == "outbound" {
            if let (Some(bytes), _cas) = self.get_shared_data("trace_id") {
                trace_id = String::from_utf8(bytes).unwrap();
            }
        } else if self.config.plugin_type == "inbound" {
            match self.set_shared_data("trace_id", Some(trace_id.to_string().as_bytes()), None) {
                Ok(_) => {
                    info!(
                        "[{}] shared context id:{}",
                        self.config.plugin_type, trace_id
                    );
                }
                Err(cause) => panic!("unexpected status: {:?}", cause),
            }
        }

        Some(Box::new(HttpFilterContext {
            trace_id,
            queue_id: self.queue_id,
            config: PluginConfig {
                plugin_type: (&*self.config.plugin_type).to_string(),
                host: None,
            },
            record: Record {
                plugin_type: "".to_string(),
                trace_id: "".to_string(),
                request_headers: vec![],
                request_body: "".to_string(),
                response_headers: vec![],
                response_body: "".to_string(),
            },
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpFilterContext {
    trace_id: String,
    config: PluginConfig,
    queue_id: u32,
    record: Record,
}

#[derive(Serialize, Deserialize)]
struct Record {
    plugin_type: String,
    trace_id: String,
    request_headers: Vec<(String, String)>,
    request_body: String,
    response_headers: Vec<(String, String)>,
    response_body: String,
}

impl Context for HttpFilterContext {}

impl HttpContext for HttpFilterContext {
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            self.record.request_body = body_str
        }
        Action::Continue
    }
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            self.record.response_body = body_str
        }
        Action::Continue
    }
    fn on_log(&mut self) {
        self.record.request_headers = self.get_http_request_headers();
        self.record.response_headers = self.get_http_response_headers();
        self.record.trace_id = (&*self.trace_id).to_string();
        self.record.plugin_type = (&*self.config.plugin_type).to_string();
        let record_json = serde_json::to_string(&self.record).expect("");
        self.enqueue_shared_queue(self.queue_id, Some(record_json.as_bytes()))
            .expect("wrong enqueue shared queue");
    }
}
