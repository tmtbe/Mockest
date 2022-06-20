use std::time::Duration;

use crate::{R_AUTHORITY, R_INBOUND_TRACE_ID, SHARED_TRACE_ID_NAME};
use log::{error, info, warn};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    plugin_type: String,
    host: String,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        return Config {
            plugin_type: self.plugin_type.clone(),
            host: self.host.clone(),
        };
    }
}
pub fn new_outbound_replay_proxy() -> Box<dyn RootContext> {
    Box::new(OutboundReplayProxy { config: None })
}
struct OutboundReplayProxy {
    config: Option<Config>,
}

impl RootContext for OutboundReplayProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            let config = serde_json::from_str(&*plugin_config).unwrap();
            self.config = Some(config);
        }
        return true;
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(OutboundReplayFilter {
            config: self.config.as_ref().unwrap().clone(),
        }))
    }
}

impl Context for OutboundReplayProxy {}

#[derive(Serialize, Deserialize)]
struct Resp {
    response_headers: Vec<(String, String)>,
    response_body: String,
}

struct OutboundReplayFilter {
    config: Config,
}

impl OutboundReplayFilter {
    fn call_replay(&mut self, body: Option<&[u8]>) {
        let host = &*self.config.host;
        let (_, authority) = self
            .get_http_request_headers()
            .iter()
            .find(|(k, _)| k == ":authority")
            .cloned()
            .expect("no authority");
        if let (Some(bytes), _cas) = self.get_shared_data(SHARED_TRACE_ID_NAME) {
            let trace_id = String::from_utf8(bytes).unwrap();
            let mut headers = self.get_http_request_headers();
            headers.push((R_INBOUND_TRACE_ID.to_string(), trace_id));
            headers.push((R_AUTHORITY.to_string(), authority));
            self.dispatch_http_call(
                host,
                headers
                    .iter()
                    .map(|(k, v)| (k.as_ref(), v.as_ref()))
                    .collect(),
                body,
                vec![],
                Duration::from_secs(2),
            )
            .expect("dispatch http call error");
        } else {
            error!("no trace id")
        }
    }
}

impl Context for OutboundReplayFilter {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(body_bytes) = self.get_http_call_response_body(0, body_size) {
            let raw_headers = self.get_http_call_response_headers();
            let headers: Vec<(&str, &str)> = raw_headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            if let Some((_, code)) = raw_headers.iter().find(|(k, _)| (return k == ":status")) {
                let resp = Resp {
                    response_headers: self.get_http_call_response_headers(),
                    response_body: base64::encode(body_bytes.clone()),
                };
                let resp_json = serde_json::to_string(&resp).expect("json error");
                self.set_property(vec!["replay"], Some(resp_json.as_ref()));
                self.send_http_response(
                    code.parse::<u32>().unwrap(),
                    headers.clone(),
                    Some(&body_bytes),
                )
            } else {
                error!("not found status code")
            }
        }
    }
}

impl HttpContext for OutboundReplayFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            self.call_replay(None);
            return Action::Pause;
        }
        Action::Continue
    }
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
                self.call_replay(Some(&body_bytes));
            }
            self.call_replay(None);
        }
        Action::Pause
    }
}
