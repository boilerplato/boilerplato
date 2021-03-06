use crate::constants;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    pub version: String,
    pub template: TemplateMeta,
    #[serde(default)]
    pub data: Vec<TemplateData>,
    pub files: Option<HashMap<String, Value>>,
    #[serde(skip)]
    pub files_map: Option<HashMap<String, CondFileMap>>,
    pub post_generate: Option<Value>,
    pub help_text: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateMeta {
    pub engine: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub extension: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TemplateData {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: TemplateDataType,
    pub values: Option<Vec<Value>>,
    #[serde(default)]
    pub message: String,
    pub required: bool,
    pub default_value: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum TemplateDataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "array[string]")]
    ArrayString,
    #[serde(rename = "array[number]")]
    ArrayNumber,
    #[serde(rename = "semver")]
    Semver,
}

#[derive(Debug, Clone)]
pub struct CondFileMap {
    pub check: String,
    pub new_name: Option<String>,
}

#[derive(Debug)]
pub enum ConfigFileType {
    JSON,
    YAML,
}

impl TemplateDataType {
    pub fn default_value(&self) -> Value {
        match self {
            TemplateDataType::String => Value::String(String::default()),
            TemplateDataType::Number => Value::Number(Number::from_f64(0_f64).unwrap()),
            TemplateDataType::Bool => Value::Bool(bool::default()),
            TemplateDataType::ArrayString => Value::Array(Vec::default()),
            TemplateDataType::ArrayNumber => Value::Array(Vec::default()),
            TemplateDataType::Semver => Value::String(constants::TEMPLATE_TYPE_SEMVER_DEFAULT_VALUE.to_owned()),
        }
    }
}

impl TemplateConfig {
    pub fn parse(config_text: &str, config_file_type: ConfigFileType) -> crate::Result<TemplateConfig> {
        match config_file_type {
            ConfigFileType::JSON => serde_json::from_str::<TemplateConfig>(config_text).wrap(),
            ConfigFileType::YAML => serde_yaml::from_str::<TemplateConfig>(config_text).wrap(),
        }
    }
}
