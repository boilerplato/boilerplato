use crate::constants;
use crate::generator::post_generator::gen_extra_template_data;
use crate::prelude::*;
use crate::utils::json_val_to_actual_str;
use colored::*;
use serde_json::Value;
use std::collections::HashMap;
use std::env::consts::OS;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn handle_post_generate_command(
    val: &Value,
    template_dir: &Path,
    template_source_dir: &Path,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
) -> crate::Result<()> {
    match val {
        Value::String(_) | Value::Array(_) => {
            execute_commands(val, template_dir, template_source_dir, project_dir, template_data)?
        }
        Value::Object(ref commands_map) => {
            if let Some(val) = commands_map.get(constants::TEMPLATE_OS_FLAG_ALL) {
                execute_commands(val, template_dir, template_source_dir, project_dir, template_data)?;
            }
            if let Some(val) = commands_map.get(OS) {
                execute_commands(val, template_dir, template_source_dir, project_dir, template_data)?;
            }
        }
        _ => (),
    }

    Ok(())
}

fn execute_commands(
    val: &Value,
    template_dir: &Path,
    template_source_dir: &Path,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
) -> crate::Result<()> {
    match val {
        Value::String(ref command) => execute_single_command(
            command.as_str(),
            template_dir,
            template_source_dir,
            project_dir,
            template_data,
        )?,
        Value::Array(ref commands) => {
            for command in commands.iter().filter_map(|v| v.as_str()) {
                execute_single_command(command, template_dir, template_source_dir, project_dir, template_data)?
            }
        }
        _ => (),
    }

    Ok(())
}

fn execute_single_command(
    command: &str,
    template_dir: &Path,
    template_source_dir: &Path,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
) -> crate::Result<()> {
    let command = command.trim();

    let envs = template_data
        .iter()
        .map(|(key, val)| (key.to_string(), json_val_to_actual_str(val)))
        .chain(gen_extra_template_data(template_dir, template_source_dir, project_dir));

    let (program, args) = if OS == "windows" {
        ("cmd", ["/C", command])
    } else {
        ("sh", ["-c", command])
    };

    println!("{} {}", "$".bold(), command.green());
    Command::new(program)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(project_dir)
        .envs(envs)
        .status()
        .context(format!("Couldn't run the post generate command: '{}'", command))?;

    Ok(())
}
