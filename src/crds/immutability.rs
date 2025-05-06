use schemars;

pub fn immutable_string(_: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "string",
        "x-kubernetes-validations": [{
            "rule": "self == oldSelf",
            "message": "field is immutable"
        }]
    }))
    .unwrap()
}