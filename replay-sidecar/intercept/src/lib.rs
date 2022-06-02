mod sony_flake;

use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

const COLLECTOR_SERVICE_UPSTREAM: &str = "collector-service";

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(PluginContext {
            config: PluginConfig{
                plugin_type:"none".to_string(),
                host:None,
                post_path:None,
            }
        })
    });
}}

#[derive(Serialize, Deserialize)]
struct PluginConfig {
    plugin_type: String,
    host: Option<String>,
    post_path: Option<String>,
}
struct PluginContext {
    config: PluginConfig,
}

impl Context for PluginContext {}

impl RootContext for PluginContext {
    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(plugin_config_bytes) = self.get_plugin_configuration() {
            let plugin_config = String::from_utf8(plugin_config_bytes).unwrap();
            self.config = serde_json::from_str(&*plugin_config).unwrap();
            info!("[{}] plugin started", self.config.plugin_type)
        }
        true
    }
    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        info!(
            "[{}] http context started, context_id:{}",
            self.config.plugin_type, context_id
        );
        let host = self.config.host.as_ref().unwrap();
        let post_path = self.config.post_path.as_ref().unwrap();
        Some(Box::new(HttpFilterContext {
            config: PluginConfig {
                plugin_type: (&*self.config.plugin_type).to_string(),
                host: Some(host.to_string()),
                post_path: Some(post_path.to_string()),
            },
            request: Request {
                request_headers: vec![],
                request_body: "".to_string(),
            },
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}
#[derive(Serialize, Deserialize)]
struct Request {
    request_headers: Vec<(String, String)>,
    request_body: String,
}
#[derive(Serialize, Deserialize)]
struct Response {
    response_headers: Vec<(String, String)>,
    response_body: String,
}

struct HttpFilterContext {
    config: PluginConfig,
    request: Request,
}

impl Context for HttpFilterContext {
    fn on_http_call_response(
        &mut self,
        _token_id: u32,
        _num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        let response: Response;
        if let Some(body_bytes) = self.get_http_call_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            response = serde_json::from_str(&*body_str).expect("json error");
            let headers: Vec<(&str, &str)> = response
                .response_headers
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref()))
                .collect();
            let status_code = &response
                .response_headers
                .iter()
                .find(|(k, _v)| k == ":status")
                .unwrap()
                .1
                .parse::<u32>()
                .unwrap();
            let mut body: Option<&[u8]> = None;
            if !response.response_body.is_empty() {
                body = Some(response.response_body.as_bytes());
            }
            self.send_http_response(*status_code, headers, body);
        } else {
            self.send_http_response(
                403,
                vec![("Powered-By", "proxy-wasm")],
                Some(b"Access forbidden.\n"),
            );
        }
    }
}

impl HttpContext for HttpFilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, end_of_stream: bool) -> Action {
        self.request.request_headers = self.get_http_request_headers();
        if end_of_stream {
            self.call_collector();
            Action::Pause
        } else {
            Action::Continue
        }
    }
    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if end_of_stream {
            if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
                let body_str = String::from_utf8(body_bytes).unwrap();
                self.request.request_body = body_str
            }
            self.call_collector();
        }
        Action::Pause
    }
}
impl HttpFilterContext {
    fn call_collector(&mut self) {
        let post_path = self.config.post_path.as_ref().unwrap();
        let host = self.config.host.as_ref().unwrap();
        let record_json = serde_json::to_string(&self.request).expect("json error");
        info!("{}", record_json);
        self.dispatch_http_call(
            COLLECTOR_SERVICE_UPSTREAM,
            vec![
                (":method", "POST"),
                (":path", post_path.as_str()),
                (":authority", host.as_str()),
            ],
            Option::Some(record_json.as_ref()),
            vec![],
            Duration::from_secs(2),
        )
        .expect("dispatch http call error");
    }
}
