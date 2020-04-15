use crate::constants;
use crate::generator::post_generator::gen_extra_template_data;
use crate::generator::post_generator::var_subs::substitute_variable_in_text;
use crate::prelude::*;
use crate::template_engine::TemplateEngine;
use serde_json::Value;
use std::collections::HashMap;
use std::env::consts::OS;
use std::path::Path;

pub fn handle_post_generate_help_text(
    val: &Value,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
    template_engine: &TemplateEngine,
) -> crate::Result<()> {
    match val {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Array(_) => {
            print_help_texts(val, project_dir, template_data, template_engine)?
        }
        Value::Object(ref texts_map) => {
            if let Some(val) = texts_map.get(constants::TEMPLATE_OS_FLAG_ALL) {
                print_help_texts(val, project_dir, template_data, template_engine)?;
            }
            if let Some(val) = texts_map.get(OS) {
                print_help_texts(val, project_dir, template_data, template_engine)?;
            }
        }
        Value::Null => (),
    }

    Ok(())
}

fn print_help_texts(
    val: &Value,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
    template_engine: &TemplateEngine,
) -> crate::Result<()> {
    match val {
        Value::String(ref text) => print_single_help_text(text.as_str(), project_dir, template_data, template_engine)?,
        Value::Number(ref num) => {
            print_single_help_text(num.to_string().as_str(), project_dir, template_data, template_engine)?
        }
        Value::Bool(ref b) => {
            print_single_help_text(b.to_string().as_str(), project_dir, template_data, template_engine)?
        }
        Value::Array(ref values) => {
            for val in values.iter() {
                print_help_texts(val, project_dir, template_data, template_engine)?;
            }
        }
        _ => (),
    }

    Ok(())
}

fn print_single_help_text(
    text: &str,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
    template_engine: &TemplateEngine,
) -> crate::Result<()> {
    let extra_data = gen_extra_template_data(project_dir);
    let text = substitute_variable_in_text(text, template_data, &extra_data);

    let rendered_text = template_engine
        .render_template(text.as_str(), template_data)
        .context("Failed to generate help text from handlebars format")?;

    println!("{}", rendered_text);

    Ok(())
}
