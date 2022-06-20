use std::time::Duration;

use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Action;
use serde::{Deserialize, Serialize};

use crate::SHARED_TRACE_ID_NAME;

#[derive(Serialize, Deserialize)]
struct Config {
    plugin_type: String,
    host: String,
    path: String,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        return Config {
            plugin_type: self.plugin_type.clone(),
            host: self.host.clone(),
            path: self.path.clone(),
        };
    }
}
pub fn new_inbound_record_proxy() -> Box<dyn RootContext> {
    Box::new(InboundRecordProxy { config: None })
}
struct InboundRecordProxy {
    config: Option<Config>,
}

impl RootContext for InboundRecordProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            let config = serde_json::from_str(&*plugin_config).unwrap();
            self.config = Some(config);
        }
        return true;
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(InboundRecordFilter {
            config: self.config.as_ref().unwrap().clone(),
            record: Record {
                plugin_type: None,
                trace_id: None,
                request_headers: None,
                request_body: None,
                response_headers: None,
                response_body: None,
            },
        }))
    }
}

impl Context for InboundRecordProxy {}

#[derive(Serialize, Deserialize)]
struct Record {
    plugin_type: Option<String>,
    trace_id: Option<String>,
    request_headers: Option<Vec<(String, String)>>,
    request_body: Option<String>,
    response_headers: Option<Vec<(String, String)>>,
    response_body: Option<String>,
}

struct InboundRecordFilter {
    config: Config,
    record: Record,
}
impl InboundRecordFilter {
    fn call_collector(&self) {
        let host = self.config.host.as_str();
        let path = self.config.path.as_str();
        let record_json = serde_json::to_string(&self.record).expect("json error");
        self.dispatch_http_call(
            host,
            vec![(":method", "POST"), (":path", path), (":authority", host)],
            Option::Some(record_json.as_ref()),
            vec![],
            Duration::from_secs(2),
        )
        .expect("dispatch http error");
    }
}
impl Context for InboundRecordFilter {}

impl HttpContext for InboundRecordFilter {
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            let body = base64::encode(&body_bytes);
            self.record.request_body = Some(body)
        }
        Action::Continue
    }
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body = base64::encode(&body_bytes);
            self.record.response_body = Some(body)
        }
        Action::Continue
    }
    fn on_log(&mut self) {
        let mut trace_id = None;
        if let (Some(bytes), _cas) = self.get_shared_data(SHARED_TRACE_ID_NAME) {
            trace_id = Some(String::from_utf8(bytes).unwrap());
        }
        self.record.request_headers = Some(self.get_http_request_headers());
        self.record.response_headers = Some(self.get_http_response_headers());
        self.record.trace_id = trace_id;
        self.record.plugin_type = Some((&*self.config.plugin_type).to_string());
        self.call_collector()
    }
}
