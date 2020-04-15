use crate::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub fn handle_post_generate_help_text(
    val: &Value,
    project_dir: &Path,
    template_data: &HashMap<&str, Value>,
) -> crate::Result<()> {
    todo!()
}
