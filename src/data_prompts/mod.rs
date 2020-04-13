use crate::constants;
use crate::prelude::*;
use crate::types::{TemplateConfig, TemplateData, TemplateDataType};
use colored::*;
use semver::Version;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

pub fn ask_data(config: &TemplateConfig) -> crate::Result<HashMap<&str, Value>> {
    let mut template_data = HashMap::with_capacity(config.data.len());

    for d in config.data.iter() {
        // if d.name == constants::TEMPLATE_DATA_APP_NAME {
        //     template_data.insert(
        //         d.name.as_str(),
        //         d.default_value.clone().unwrap_or_else(|| d.data_type.default_value()),
        //     );
        // }
        template_data.insert(d.name.as_str(), ask_a_single_data(d)?);
    }

    Ok(template_data)
}

pub fn ask_a_single_data(data_config: &TemplateData) -> crate::Result<Value> {
    let msg = get_data_massage(data_config);

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut r_handle = stdin.lock();
    let mut w_handle = stdout.lock();

    loop {
        w_handle.write_fmt(format_args!("{}", "? ".cyan())).wrap()?;
        w_handle.write_fmt(format_args!("{}", msg.as_str().bold())).wrap()?;
        w_handle.write_fmt(format_args!("{}", " › ".bright_black())).wrap()?;
        w_handle.flush().wrap()?;

        let mut line = String::with_capacity(10);
        r_handle.read_line(&mut line).wrap()?;

        let validation = validate_input(data_config, line.as_str());
        match validation {
            Ok(val) => {
                return Ok(val);
            }
            Err(err) => {
                w_handle.write_fmt(format_args!("› ")).wrap()?;
                w_handle
                    .write_fmt(format_args!("{}", err.to_string().as_str().red().italic()))
                    .wrap()?;
                w_handle.write_fmt(format_args!("\n")).wrap()?;
                w_handle.flush().wrap()?;
            }
        }
    }
}

pub fn get_data_massage(data_config: &TemplateData) -> String {
    let mut msg = data_config.message.trim();

    let default_msg = format!("Enter {}: ", data_config.name);
    if msg.is_empty() {
        msg = default_msg.as_str();
    }

    if data_config.required {
        return format!("{}", msg);
    }

    if let Some(ref value) = data_config.default_value {
        match data_config.data_type {
            TemplateDataType::String => {
                let value = value.as_str().unwrap_or("").trim();
                if value.is_empty() {
                    format!("{}", msg)
                } else {
                    format!("{} ({})", msg, value)
                }
            }
            TemplateDataType::Number => format!("{} ({})", msg, value.as_f64().unwrap_or(0_f64)),
            TemplateDataType::Bool => {
                let bool_str = if value.as_bool().unwrap_or(false) { "yes" } else { "no" };
                format!("{} ({})", msg, bool_str)
            }
            TemplateDataType::ArrayString => {
                let arr_str = value
                    .as_array()
                    .into_iter()
                    .map(|x| x.iter())
                    .flatten()
                    .map(|x: &Value| x.as_str().unwrap_or(""))
                    .collect::<Vec<&str>>()
                    .join(", ");

                if arr_str.is_empty() {
                    format!("{}", msg)
                } else {
                    format!("{} ([{}])", msg, arr_str)
                }
            }
            TemplateDataType::ArrayNumber => {
                let arr_str = value
                    .as_array()
                    .into_iter()
                    .map(|x| x.iter())
                    .flatten()
                    .map(|x: &Value| x.as_f64().unwrap_or(0_f64).to_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                if arr_str.is_empty() {
                    format!("{}", msg)
                } else {
                    format!("{} ([{}])", msg, arr_str)
                }
            }
            TemplateDataType::Semver => {
                let value = value.as_str().unwrap_or("").trim();
                if value.is_empty() {
                    format!("{}", value)
                } else {
                    format!("{} ({})", msg, value)
                }
            }
        }
    } else {
        format!("{}", msg)
    }
}

fn validate_input(data_config: &TemplateData, input: &str) -> crate::Result<Value> {
    let input = input.trim();

    if input.is_empty() {
        if data_config.required {
            return Err(crate::Error::new("Value can't be empty"));
        } else {
            return Ok(data_config
                .default_value
                .clone()
                .unwrap_or_else(|| data_config.data_type.default_value()));
        }
    }

    match data_config.data_type {
        TemplateDataType::String => {
            let value = Value::String(input.to_owned());
            if let Some(ref values) = data_config.values {
                if values.contains(&value) {
                    Ok(value)
                } else {
                    Err(crate::Error::new(format!(
                        "Value must be one of: {}",
                        values
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<&str>>()
                            .join(", ")
                    )))
                }
            } else {
                Ok(value)
            }
        }
        TemplateDataType::Number => {
            let value = input
                .parse::<f64>()
                .map_err(|_| crate::Error::new("Value must be a number"))
                .and_then(|v| Number::from_f64(v).ok_or_else(|| crate::Error::new("Value must be a number")))
                .map(|v| Value::Number(v))?;

            if let Some(ref values) = data_config.values {
                let values = values
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .filter_map(|v| Number::from_f64(v))
                    .map(|v| Value::Number(v))
                    .collect::<Vec<Value>>();

                if values.contains(&value) {
                    Ok(value)
                } else {
                    Err(crate::Error::new(format!(
                        "Value must be one of: {}",
                        values
                            .iter()
                            .filter_map(|v| v.as_f64())
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )))
                }
            } else {
                Ok(value)
            }
        }
        TemplateDataType::Bool => {
            if constants::TEMPLATE_TYPE_BOOL_POSSIBLE_TRUTHY_INPUTS.contains(&input.to_lowercase().as_str()) {
                Ok(Value::Bool(true))
            } else if constants::TEMPLATE_TYPE_BOOL_POSSIBLE_FALSY_INPUTS.contains(&input.to_lowercase().as_str()) {
                Ok(Value::Bool(false))
            } else {
                Err(crate::Error::new(format!(
                    "Value must be one of: {}, {}",
                    constants::TEMPLATE_TYPE_BOOL_POSSIBLE_TRUTHY_INPUTS.join(", "),
                    constants::TEMPLATE_TYPE_BOOL_POSSIBLE_FALSY_INPUTS.join(", ")
                )))
            }
        }
        TemplateDataType::ArrayString => Ok(Value::Array(
            constants::RE_COMMA_SEPARATOR
                .split(input)
                .map(|p| Value::String(p.trim().to_owned()))
                .collect(),
        )),
        TemplateDataType::ArrayNumber => Ok(Value::Array(
            constants::RE_COMMA_SEPARATOR
                .split(input)
                .filter_map(|p| p.trim().parse::<f64>().ok())
                .filter_map(|v| Number::from_f64(v))
                .map(|p| Value::Number(p))
                .collect(),
        )),
        TemplateDataType::Semver => input
            .parse::<Version>()
            .map(|v| Value::String(v.to_string()))
            .map_err(|_| crate::Error::new("Value must be a semver e.g. 1.0.0"))
            .wrap(),
    }
}

// pub data_type: TemplateDataType,
// pub values: Option<Vec<Value>>,
// pub required: bool,

// pub name: String,
// #[serde(rename = "type")]
// pub data_type: TemplateDataType,
// pub values: Option<Vec<Value>>,
// #[serde(default)]
// pub message: String,
// pub required: bool,
// pub default_value: Option<Value>,

//
// version: 1.0.0
// templateEngine: handlebars
// data:
// - name: appName
// type: string
// message: "Enter your app name: "
// required: true
// - name: anyNumber
// type: number
// message: "Enter any number: "
// required: false
// defaultValue: 0
// - name: isPrivate
// type: bool
// message: "Is it a private project?"
// required: false
// defaultValue: false
// - name: listValue
// type: array[number]
// message: "Enter multiple numbers: "
// required: true
// - name: listValue2
// type: array[string]
// message: "Enter multiple strings: "
// required: true
// - name: license
// type: string
// values:
// - MIT
// - APACHE
// message: "Choose license [MIT, APACHE]: "
// required: false
// defaultValue: MIT
// - name: chooseNumberOneToFive
// type: number
// values:
// - 1
// - 2
// - 3
// - 4
// - 5
// message: "Choose a number from 1 to 5: "
// required: false
// defaultValue: 1
// - name: appVersion
// type: semver
// message: "Please enter initial app version: "
// required: false
// defaultValue: "1.0.0"
// postGenerate:
// - "npm install"
// helpText: >-
// This is a very long sentence
// that spans several lines in the YAML
// but which will be rendered as a string
// with NO carriage returns.
//
