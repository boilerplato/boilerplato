use crate::constants;
use std::collections::HashMap;
use std::env;
use std::path::Path;

pub use self::help_text::handle_post_generate_help_text;
pub use self::post_command::handle_post_generate_command;

mod help_text;
mod post_command;
mod var_subs;

fn gen_extra_template_data(project_dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert(
        constants::TEMPLATE_EXTRA_VAR_PROJECT_DIR_FULL_PATH.to_owned(),
        project_dir.to_str().map(|p| p.to_owned()).unwrap_or(String::new()),
    );

    map.insert(
        constants::TEMPLATE_EXTRA_VAR_PROJECT_DIR_REL_PATH.to_owned(),
        env::current_dir()
            .ok()
            .and_then(|cwd| pathdiff::diff_paths(project_dir, cwd.as_path()))
            .and_then(|p| p.to_str().map(|p| p.to_owned()))
            .unwrap_or(String::new()),
    );

    map
}
