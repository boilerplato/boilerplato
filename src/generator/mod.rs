use crate::generator::project_template::ProjectTemplate;
use crate::prelude::*;
use std::fs;
use std::path::Path;

mod post_generator;
mod project_template;

pub fn gen_source_code_from_template<P: AsRef<Path>, T: AsRef<str>>(project_dir: P, template: T) -> crate::Result<()> {
    if let Err(err) = fs::create_dir_all(project_dir.as_ref()) {
        return Err(crate::Error::new(format!(
            "Couldn't create the project directory: {}",
            err
        )));
    }

    ProjectTemplate::parse(template).disburse(
        project_dir
            .as_ref()
            .canonicalize()
            .context("Couldn't get the absolute project path")?,
    )
}
