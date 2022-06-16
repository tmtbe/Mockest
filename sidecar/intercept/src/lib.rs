mod sony_flake;

use base64::{decode, encode};
use log::{info, warn};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use sony_flake::SonyFlakeEntity;
use std::time::{Duration, SystemTime};

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

struct HttpFilterContext {
    trace_id: String,
    config: PluginConfig,
    queue_id: u32,
    record: Record,
}

#[derive(Serialize, Deserialize)]
struct Record {
    plugin_type: Option<String>,
    trace_id: Option<String>,
    request_headers: Option<Vec<(String, String)>>,
    request_body: Option<String>,
    response_headers: Vec<(String, String)>,
    response_body: String,
}

impl Context for HttpFilterContext {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(body_bytes) = self.get_http_call_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            let record: Record;
            info!("{}", body_str);
            record = serde_json::from_str(&*body_str).expect("json error");
            self.record.response_headers = record.response_headers.clone();
            self.record.response_body = record.response_body.clone();
            let headers: Vec<(&str, &str)> = record
                .response_headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            let body = base64::decode(record.response_body).expect("base64 error");
            if let Some((_, code)) = record
                .response_headers
                .iter()
                .find(|(k, _)| (return k == ":status"))
            {
                self.send_http_response(code.parse::<u32>().unwrap(), headers, Some(body.as_ref()))
            } else {
                warn!("could not find status code from headers: {}", body_str);
                self.send_http_response(500, headers, Some(body.as_ref()))
            }
        }
    }
}

impl HttpContext for HttpFilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        if end_of_stream && self.config.plugin_type == OUTBOUND_REPLAY {
            self.call_replay();
            return Action::Pause;
        }
        Action::Continue
    }
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            let body = base64::encode(&body_bytes);
            self.record.request_body = Some(body)
        }
        if self.config.plugin_type == OUTBOUND_REPLAY {
            self.call_replay();
            return Action::Pause;
        }
        Action::Continue
    }
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body = base64::encode(&body_bytes);
            self.record.response_body = body
        }
        Action::Continue
    }
    fn on_log(&mut self) {
        self.record.request_headers = Some(self.get_http_request_headers());
        self.record.response_headers = self.get_http_response_headers();
        self.record.trace_id = Some((&*self.trace_id).to_string());
        self.record.plugin_type = Some((&*self.config.plugin_type).to_string());
        let record_json = serde_json::to_string(&self.record).expect("json error");
        self.enqueue_shared_queue(self.queue_id, Some(record_json.as_bytes()))
            .expect("wrong enqueue shared queue");
    }
}

impl HttpFilterContext {
    fn call_replay(&mut self) {
        let host = self.config.host.as_ref().unwrap();
        let replay_path = self.config.replay_path.as_ref().unwrap();
        let record_json = serde_json::to_string(&self.record).expect("json error");
        self.dispatch_http_call(
            COLLECTOR_SERVICE_UPSTREAM,
            vec![
                (":method", "POST"),
                (":path", replay_path.as_str()),
                (":authority", host.as_str()),
            ],
            Some(record_json.as_ref()),
            vec![],
            Duration::from_secs(2),
        )
        .expect("dispatch http call error");
    }
}
