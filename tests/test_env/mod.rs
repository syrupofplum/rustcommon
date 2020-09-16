use std::collections::HashMap;
use std::fs;
use serde_json;

pub fn init() -> Option<HashMap<String, String>> {
    let content = fs::read_to_string("test_env.config").expect("{}");
    let env_val: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
    let mut env_map: HashMap<String, String> = HashMap::new();
    for (key, val) in env_val.as_object().unwrap() {
        env_map.insert(key.clone(), val.to_string());
    }
    Some(env_map)
}