mod sony_flake;

use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use sony_flake::SonyFlakeEntity;
use std::time::Duration;

const COLLECTOR_SERVICE_UPSTREAM: &str = "collector-service";
const OUTBOUND: &str = "outbound";
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
                post_path:None,
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
    post_path: Option<String>,
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
            let post_path = self.config.post_path.as_ref().unwrap();
            self.dispatch_http_call(
                COLLECTOR_SERVICE_UPSTREAM,
                vec![
                    (":method", "POST"),
                    (":path", post_path.as_str()),
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
        if self.config.plugin_type == OUTBOUND {
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

        Some(Box::new(HttpFilterContext {
            trace_id,
            queue_id: self.queue_id,
            config: PluginConfig {
                plugin_type: (&*self.config.plugin_type).to_string(),
                host: None,
                post_path: None,
            },
            record: Record {
                plugin_type: "".to_string(),
                trace_id: "".to_string(),
                request_headers: vec![],
                request_body: vec![],
                response_headers: vec![],
                response_body: vec![],
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
    request_body: Bytes,
    response_headers: Vec<(String, String)>,
    response_body: Bytes,
}

impl Context for HttpFilterContext {}

impl HttpContext for HttpFilterContext {
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            self.record.request_body = body_bytes
        }
        Action::Continue
    }
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            self.record.response_body = body_bytes
        }
        Action::Continue
    }
    fn on_log(&mut self) {
        self.record.request_headers = self.get_http_request_headers();
        self.record.response_headers = self.get_http_response_headers();
        self.record.trace_id = (&*self.trace_id).to_string();
        self.record.plugin_type = (&*self.config.plugin_type).to_string();
        let record_json = serde_json::to_string(&self.record).expect("json error");
        self.enqueue_shared_queue(self.queue_id, Some(record_json.as_bytes()))
            .expect("wrong enqueue shared queue");
    }
}
