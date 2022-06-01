trait RecordWrite {
    fn add_record_json(&self, _json: String) {}
}
struct RecordWithFile {
    file_name: String,
}
impl RecordWrite for RecordWithFile {
    fn add_record_json(&self, json: String) {}
}
