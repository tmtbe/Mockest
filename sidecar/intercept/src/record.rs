use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Record {
    pub(crate) plugin_type: Option<String>,
    pub(crate) trace_id: Option<String>,
    pub(crate) request_headers: Option<Vec<(String, String)>>,
    pub(crate) request_body: Option<String>,
    pub(crate) response_headers: Vec<(String, String)>,
    pub(crate) response_body: String,
}