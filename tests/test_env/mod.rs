use std::collections::HashMap;
use std::fs;
use serde_json;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ENV_CONFIG: HashMap<String, String> = {
        let content = fs::read_to_string("test_env.config").unwrap_or(String::from("{}"));
        let env_val: serde_json::Value = serde_json::from_str(content.as_str()).unwrap();
        let mut env_map: HashMap<String, String> = HashMap::new();
        for (key, val) in env_val.as_object().unwrap() {
            env_map.insert(key.clone(), val.as_str().unwrap().to_string());
        }
        env_map
    };
}

pub fn init() {

}
