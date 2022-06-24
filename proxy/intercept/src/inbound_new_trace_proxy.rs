use std::time::Duration;

use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Action;
use serde::{Deserialize, Serialize};

use crate::sony_flake::SonyFlakeEntity;
use crate::SHARED_TRACE_ID_NAME;

struct InboundNewTraceProxy {}

pub fn new_inbound_new_trace_proxy() -> Box<dyn RootContext> {
    Box::new(InboundNewTraceProxy {})
}

impl RootContext for InboundNewTraceProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        return true;
    }
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(InboundNewTraceFilter {}))
    }
}

impl Context for InboundNewTraceProxy {}

struct InboundNewTraceFilter {}
impl InboundNewTraceFilter {
    fn new_trace(&self) {
        let trace_id: String = SonyFlakeEntity::new_default()
            .get_id(self.get_current_time())
            .to_string();
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
    }
}
impl Context for InboundNewTraceFilter {}

impl HttpContext for InboundNewTraceFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            self.new_trace();
        }
        Action::Continue
    }
    fn on_http_request_body(&mut self, _body_size: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            self.new_trace();
        }
        Action::Continue
    }
}
