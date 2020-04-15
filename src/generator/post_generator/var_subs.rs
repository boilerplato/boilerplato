use crate::utils;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde_json::Value;
use std::collections::HashMap;
use std::env;

lazy_static! {
    static ref VARIABLE_RE: Regex = Regex::new(r"(?:\$([a-zA-Z\d_]+))|(:?\$\{([a-zA-Z\d_]+)\})").unwrap();
}

pub fn substitute_variable_in_text(
    text: &str,
    template_data: &HashMap<&str, Value>,
    extra_data: &HashMap<String, String>,
) -> String {
    VARIABLE_RE
        .replace_all(text, |caps: &Captures| {
            let var_name = caps
                .get(1)
                .or_else(|| caps.get(2))
                .expect("Failed to extract variable name from text")
                .as_str();

            template_data
                .get(var_name)
                .map(|v| utils::json_val_to_actual_str(v))
                .or_else(|| extra_data.get(&var_name.to_owned()).cloned())
                .or_else(|| env::var(var_name).ok())
                .unwrap_or(String::new())
        })
        .to_string()
}
