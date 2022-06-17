use crate::COLLECTOR_SERVICE_UPSTREAM;
use log::warn;
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
            host: self.plugin_type.clone(),
            path: self.plugin_type.clone(),
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
            req: Req {
                request_headers: None,
                request_body: None,
            },
        }))
    }
}

impl Context for OutboundReplayProxy {}

#[derive(Serialize, Deserialize)]
struct Req {
    request_headers: Option<Vec<(String, String)>>,
    request_body: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Resp {
    response_headers: Vec<(String, String)>,
    response_body: String,
}

struct OutboundReplayFilter {
    config: Config,
    req: Req,
}

impl OutboundReplayFilter {
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

impl Context for OutboundReplayFilter {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        if let Some(body_bytes) = self.get_http_call_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            let resp: Resp = serde_json::from_str(&*body_str).expect("json error");
            let headers: Vec<(&str, &str)> = resp
                .response_headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            let body = base64::decode(resp.response_body).expect("base64 error");
            if let Some((_, code)) = resp
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

impl HttpContext for OutboundReplayFilter {
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
