use std::time::{Duration, SystemTime};

use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, Bytes, Status};
use serde::{Deserialize, Serialize};

use crate::{add_sign_to_record, Sign, SHARED_TRACE_ID_NAME};

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

pub fn new_outbound_record_proxy() -> Box<dyn RootContext> {
    Box::new(OutboundRecordProxy { config: None })
}

struct OutboundRecordProxy {
    config: Option<Config>,
}

impl RootContext for OutboundRecordProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            let config = serde_json::from_str(&*plugin_config).unwrap();
            self.config = Some(config);
        }
        return true;
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        let mut trace_id: String = "untracked".to_string();
        if let (Some(bytes), _cas) = self.get_shared_data(SHARED_TRACE_ID_NAME) {
            trace_id = String::from_utf8(bytes).unwrap();
        }
        Some(Box::new(OutboundRecordFilter {
            trace_id,
            config: self.config.as_ref().unwrap().clone(),
            record: Record {
                plugin_type: None,
                trace_id: None,
                request_headers: None,
                request_body: None,
                response_headers: None,
                response_body: None,
                index: 0,
            },
            context_id,
            request_body: vec![],
            response_body: vec![],
        }))
    }
}

impl Context for OutboundRecordProxy {}

#[derive(Serialize, Deserialize)]
struct Record {
    plugin_type: Option<String>,
    trace_id: Option<String>,
    request_headers: Option<Vec<(String, String)>>,
    request_body: Option<String>,
    response_headers: Option<Vec<(String, String)>>,
    response_body: Option<String>,
    index: usize,
}

#[derive(Serialize, Deserialize)]
struct Resp {
    response_headers: Vec<(String, String)>,
    response_body: String,
}

struct OutboundRecordFilter {
    trace_id: String,
    config: Config,
    record: Record,
    context_id: u32,
    request_body: Bytes,
    response_body: Bytes,
}

impl OutboundRecordFilter {
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

impl Context for OutboundRecordFilter {}

impl HttpContext for OutboundRecordFilter {
    fn on_http_request_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        if let Some(mut body_bytes) = self.get_http_request_body(0, body_size) {
            self.request_body.append(body_bytes.as_mut());
        }
        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        if let Some(mut body_bytes) = self.get_http_response_body(0, body_size) {
            self.response_body.append(body_bytes.as_mut());
        }
        Action::Continue
    }

    fn on_log(&mut self) {
        self.record.request_headers = Some(self.get_http_request_headers());
        self.record.response_headers = Some(self.get_http_response_headers());
        self.record.trace_id = Some((&*self.trace_id).to_string());
        self.record.plugin_type = Some((&*self.config.plugin_type).to_string());
        if self.response_body.len() > 0 {
            self.record.response_body = Some(base64::encode(&self.response_body))
        }
        if self.request_body.len() > 0 {
            self.record.request_body = Some(base64::encode(&self.request_body))
        }
        if let Some(body_bytes) = self.get_property(vec!["replay"]) {
            let json = String::from_utf8(body_bytes).unwrap();
            let resp: Resp = serde_json::from_str(json.as_str()).expect("json error");
            self.record.response_body = Some(resp.response_body);
            self.record.response_headers = Some(resp.response_headers);
        }
        let sign = Sign {
            request_headers: self.get_http_request_headers(),
            request_body: self.record.request_body.as_ref().map(String::to_string),
        };
        self.record.index = add_sign_to_record(sign);
        info!("outbound record");
        self.call_collector()
    }
}
