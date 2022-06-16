use crate::{PluginConfig, Record, COLLECTOR_SERVICE_UPSTREAM, OUTBOUND_REPLAY};
use log::{info, warn};
use proxy_wasm::traits::{Context, HttpContext};
use proxy_wasm::types::Action;
use std::time::Duration;

pub struct HttpFilterContext {
    pub(crate) trace_id: String,
    pub(crate) config: PluginConfig,
    pub(crate) queue_id: u32,
    pub(crate) record: Record,
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
