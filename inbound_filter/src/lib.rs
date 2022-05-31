use proxy_wasm::hostcalls::log;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Serialize, Deserialize};

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpBodyRoot) });
}}

struct HttpBodyRoot;

impl Context for HttpBodyRoot {}

impl RootContext for HttpBodyRoot {
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpBody { record: Record {
            request_headers: vec![],
            request_body: "".to_string(),
            response_headers: vec![],
            response_body: "".to_string()
        } }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
#[derive(Serialize, Deserialize)]
struct Record{
    request_headers:Vec<(String,String)>,
    request_body: String,
    response_headers:Vec<(String,String)>,
    response_body: String,
}
struct HttpBody{
    record: Record
}

impl Context for HttpBody {}

impl HttpContext for HttpBody {
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            self.record.request_body=body_str
        }
        Action::Continue
    }
    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            self.record.response_body=body_str
        }
        Action::Continue
    }
    fn on_log(&mut self) {
        self.record.request_headers = self.get_http_request_headers();
        self.record.response_headers = self.get_http_response_headers();
        let msg = serde_json::to_string(&self.record).expect("");
        log(LogLevel::Info, &*&msg).expect("");
    }
}