use std::time::Duration;

use log::info;
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Action;
use serde::{Deserialize, Serialize};

use crate::COLLECTOR_SERVICE_UPSTREAM;

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
    fn call_collector(&mut self) {
        let host = &*self.config.host;
        let path = &*self.config.path;
        let req_json = serde_json::to_string(&self.req).expect("json error");
        self.dispatch_http_call(
            COLLECTOR_SERVICE_UPSTREAM,
            vec![(":method", "POST"), (":path", path), (":authority", host)],
            Some(req_json.as_ref()),
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
        _body_size: usize,
        _num_trailers: usize,
    ) {
        self.resume_http_request()
    }
}

impl HttpContext for InboundReplayFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        self.req.request_headers = Some(self.get_http_request_headers());
        if end_of_stream {
            self.call_collector();
            return Action::Pause;
        }
        Action::Continue
    }
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
                let body = base64::encode(&body_bytes);
                self.req.request_body = Some(body)
            }
            self.call_collector();
        }
        Action::Pause
    }
}
