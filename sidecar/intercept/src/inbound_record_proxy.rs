use crate::sony_flake::SonyFlakeEntity;
use crate::{COLLECTOR_SERVICE_UPSTREAM, SHARED_DATA_NAME, SHARED_QUEUE_NAME, VM_ID};
use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Action;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
        let trace_id: String = SonyFlakeEntity::new_default()
            .get_id(self.get_current_time())
            .to_string();
        match self.set_shared_data(
            SHARED_DATA_NAME,
            Some(trace_id.to_string().as_bytes()),
            None,
        ) {
            Ok(_) => {
                info!(
                    "[{}] shared context id:{}",
                    self.config.as_ref().unwrap().plugin_type,
                    trace_id
                );
            }
            Err(cause) => panic!("unexpected status: {:?}", cause),
        }
        Some(Box::new(InboundRecordFilter {
            trace_id,
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
    trace_id: String,
    config: Config,
    record: Record,
}
impl InboundRecordFilter {
    fn call_collector(&self) {
        let host = self.config.host.as_str();
        let path = self.config.path.as_str();
        let record_json = serde_json::to_string(&self.record).expect("json error");
        match self.dispatch_http_call(
            COLLECTOR_SERVICE_UPSTREAM,
            vec![(":method", "POST"), (":path", path), (":authority", host)],
            Option::Some(record_json.as_ref()),
            vec![],
            Duration::from_secs(2),
        ) {
            Ok(e) => {
                debug!("dispatch collector api: {:?}", e)
            }
            Err(e) => {
                error!("dispatch collector api: {:?}", e)
            }
        }
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
        self.record.request_headers = Some(self.get_http_request_headers());
        self.record.response_headers = Some(self.get_http_response_headers());
        self.record.trace_id = Some((&*self.trace_id).to_string());
        self.record.plugin_type = Some((&*self.config.plugin_type).to_string());
        self.call_collector()
    }
}
