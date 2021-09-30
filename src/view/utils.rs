use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValueEntry {
    key: String,
    value: Value,
    #[serde()]
    #[serde(rename = "type")]
    type_: KeyValueType,
    group: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KeyValueType {
    String,
    Int,
    Float,
    Bool,
}

pub fn value_to_entries(value: &Value, group: &str) -> Vec<KeyValueEntry> {
    let mut entries: Vec<KeyValueEntry> = Vec::new();
    if let Some(obj) = value.as_object() {
        for (key, value) in obj {
            let mut entry = KeyValueEntry {
                key: key.clone(),
                value: value.clone(),
                type_: KeyValueType::String,
                group: group.to_string(),
            };
            if value.is_boolean() {
                entry.type_ = KeyValueType::Bool;
            } else if value.is_f64() {
                entry.type_ = KeyValueType::Float;
            } else if value.is_i64() {
                entry.type_ = KeyValueType::Int;
            } else if value.is_string() {
                entry.type_ = KeyValueType::String;
            }
            entries.push(entry);
        }
    }
    entries
}