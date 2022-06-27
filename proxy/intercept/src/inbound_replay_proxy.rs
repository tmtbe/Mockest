use std::time::Duration;

use crate::{R_AUTHORITY, R_INBOUND_TRACE_ID, R_MATCH_INBOUND, R_MATCH_TYPE, SHARED_TRACE_ID_NAME};
use log::info;
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::{Action, Bytes};
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
pub fn new_inbound_replay_proxy() -> Box<dyn RootContext> {
    Box::new(InboundReplayProxy { config: None })
}
struct InboundReplayProxy {
    config: Option<Config>,
}

impl RootContext for InboundReplayProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            let config = serde_json::from_str(&*plugin_config).unwrap();
            self.config = Some(config);
        }
        return true;
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(InboundReplayFilter {
            config: self.config.as_ref().unwrap().clone(),
            req: Req {
                request_headers: None,
                request_body: None,
            },
        }))
    }
}

impl Context for InboundReplayProxy {}

#[derive(Serialize, Deserialize)]
struct Req {
    request_headers: Option<Vec<(String, String)>>,
    request_body: Option<String>,
}

struct InboundReplayFilter {
    config: Config,
    req: Req,
}

impl InboundReplayFilter {
    fn call_collector(&mut self, body: Bytes) {
        let host = &*self.config.host;
        let mut headers = self.req.request_headers.as_ref().unwrap().clone();
        headers.push((R_MATCH_TYPE.to_string(), R_MATCH_INBOUND.to_string()));
        let mut dispatch_body: Option<&[u8]> = None;
        if body.len() != 0 {
            dispatch_body = Some(&body);
        }
        info!(
            "[inbound_replay] call: {}, body size: {}",
            serde_json::to_string(&headers).unwrap(),
            body.len()
        );
        self.dispatch_http_call(
            host,
            headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect(),
            dispatch_body,
            vec![],
            Duration::from_secs(2),
        )
        .expect("dispatch http call error");
    }
}

impl Context for InboundReplayFilter {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(trace_id) = self.get_http_call_response_header(R_INBOUND_TRACE_ID) {
            match self.set_shared_data(
                SHARED_TRACE_ID_NAME,
                Some(trace_id.to_string().as_bytes()),
                None,
            ) {
                Ok(_) => {
                    info!("new trace:{}", trace_id);
                }
                Err(cause) => panic!("unexpected status: {:?}", cause),
            }
            self.resume_http_request()
        } else {
            self.send_http_response(
                500,
                vec![],
                Some("[sidecar] inbound proxy get trace id error".as_ref()),
            )
        }
    }
}

impl HttpContext for InboundReplayFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        self.req.request_headers = Some(self.get_http_request_headers());
        if end_of_stream {
            self.call_collector(None);
            return Action::Pause;
        }
        Action::Continue
    }
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
                let body = base64::encode(&body_bytes);
                self.req.request_body = Some(body);
                self.call_collector(Some(&body_bytes));
            } else {
                self.call_collector(None);
            }
        }
        Action::Pause
    }
}
