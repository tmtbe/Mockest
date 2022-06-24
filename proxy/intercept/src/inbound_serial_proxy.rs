use std::time::Duration;

use log::{debug, error, info};
use proxy_wasm::traits::{Context, HttpContext, RootContext};
use proxy_wasm::types::Status::Empty;
use proxy_wasm::types::{Action, Bytes};
use serde::{Deserialize, Serialize};

const LOCK_SHARE_KEY: &str = "lock";
const LOCKED: &str = "locked";
const UNLOCK: &str = "unlock";
struct InboundSerialProxy {}

pub fn new_inbound_serial_proxy() -> Box<dyn RootContext> {
    Box::new(InboundSerialProxy {})
}

impl RootContext for InboundSerialProxy {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        return true;
    }
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(InboundNewTraceFilter {}))
    }
}

impl Context for InboundSerialProxy {}

struct InboundNewTraceFilter {}
impl InboundNewTraceFilter {
    fn unlock(&self) {
        match self.set_shared_data(LOCK_SHARE_KEY, Some(UNLOCK.as_ref()), None) {
            Ok(_) => {}
            Err(_cause) => {}
        }
    }
    fn lock(&self) {
        loop {
            let (has_lock, cas) = self.get_shared_data(LOCK_SHARE_KEY);
            if has_lock.is_none() || String::from_utf8(has_lock.unwrap()).unwrap() == UNLOCK {
                if cas.is_none() {
                    match self.set_shared_data(LOCK_SHARE_KEY, Some(LOCKED.as_ref()), Some(1)) {
                        Ok(_) => break,
                        Err(_cause) => {}
                    }
                } else {
                    match self.set_shared_data(LOCK_SHARE_KEY, Some(LOCKED.as_ref()), cas) {
                        Ok(_) => break,
                        Err(_cause) => {}
                    }
                }
            }
        }
    }
}
impl Context for InboundNewTraceFilter {}

impl HttpContext for InboundNewTraceFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.lock();
        Action::Continue
    }
    fn on_log(&mut self) {
        self.unlock();
    }
}
