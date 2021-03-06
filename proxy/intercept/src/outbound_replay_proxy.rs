use std::time::Duration;

use crate::{
    add_sign_to_replay, Sign, R_AUTHORITY, R_INBOUND_TRACE_ID, R_INDEX, R_MATCH_OUTBOUND,
    R_MATCH_TYPE, SHARED_TRACE_ID_NAME,
};
use log::{error, info, warn};
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
            request_headers: vec![],
            request_body: vec![],
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
    request_headers: Vec<(String, String)>,
    request_body: Bytes,
}

impl OutboundReplayFilter {
    fn call_replay(&mut self, index: usize) {
        let host = &*self.config.host;
        let (_, authority) = self
            .request_headers
            .clone()
            .iter()
            .find(|(k, _)| k == ":authority")
            .cloned()
            .expect("no authority");
        let mut dispatch_body: Option<&[u8]> = None;
        if self.request_body.len() != 0 {
            dispatch_body = Some(&self.request_body);
        }
        let mut headers: Vec<(String, String)> = vec![];
        for x in self.request_headers.clone() {
            if x.0.to_lowercase().starts_with("x-") {
                continue;
            }
            if x.0.to_lowercase() == "content-length" {
                continue;
            }
            if x.0 == ":scheme" {
                headers.push((":scheme".to_string(), "http".to_string()));
            } else if x.0 == ":authority" {
                headers.push((":authority".to_string(), host.to_string()));
            } else {
                headers.push(x);
            }
        }
        if let (Some(bytes), _cas) = self.get_shared_data(SHARED_TRACE_ID_NAME) {
            let trace_id = String::from_utf8(bytes).unwrap();
            headers.push((R_INBOUND_TRACE_ID.to_string(), trace_id));
        } else {
            warn!("[proxy] outbound proxy not bind any trace id, you need inbound first")
        }
        headers.push((R_MATCH_TYPE.to_string(), R_MATCH_OUTBOUND.to_string()));
        headers.push((R_AUTHORITY.to_string(), authority));
        headers.push((R_INDEX.to_string(), index.to_string()));
        info!(
            "[outbound_replay] call: {}, body size: {}",
            serde_json::to_string(&headers).unwrap(),
            self.request_body.len()
        );
        self.dispatch_http_call(
            host,
            headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect(),
            dispatch_body,
            vec![],
            Duration::from_secs(5),
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
        info!("get call response");
        let raw_headers = self.get_http_call_response_headers();
        let headers: Vec<(&str, &str)> = raw_headers
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();
        if let Some((_, code)) = raw_headers.iter().find(|(k, _)| (return k == ":status")) {
            if let Some(body_bytes) = self.get_http_call_response_body(0, body_size) {
                let resp = Resp {
                    response_headers: self.get_http_call_response_headers(),
                    response_body: base64::encode(body_bytes.clone()),
                };
                let resp_json = serde_json::to_string(&resp).expect("json error");
                self.set_property(vec!["replay"], Some(resp_json.as_ref()));
                self.send_http_response(
                    code.parse::<u32>().unwrap(),
                    headers.clone(),
                    Some(&body_bytes.clone()),
                )
            } else {
                let resp = Resp {
                    response_headers: self.get_http_call_response_headers(),
                    response_body: "".to_string(),
                };
                let resp_json = serde_json::to_string(&resp).expect("json error");
                self.set_property(vec!["replay"], Some(resp_json.as_ref()));
                self.send_http_response(code.parse::<u32>().unwrap(), headers.clone(), None);
            }
        } else {
            error!("not found status code");
            self.send_http_response(
                500,
                headers.clone(),
                Some("[proxy] not found status code".as_bytes()),
            )
        }
    }
}

impl HttpContext for OutboundReplayFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.request_headers = self.get_http_request_headers();
        Action::Continue
    }
    fn on_http_request_body(&mut self, body_size: usize, _end_of_stream: bool) -> Action {
        if let Some(mut body_bytes) = self.get_http_request_body(0, body_size) {
            self.request_body.append(body_bytes.as_mut());
        }
        Action::Continue
    }
    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let mut request_body = None;
        if self.request_body.len() > 0 {
            request_body = Some(base64::encode(&self.request_body))
        }
        let sign = Sign {
            request_headers: self.request_headers.clone(),
            request_body,
        };
        self.call_replay(add_sign_to_replay(sign));
        Action::Pause
    }
}
