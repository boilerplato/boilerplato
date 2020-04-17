use crate::constants;
use crate::data_prompts;
use crate::generator::post_generator::{
    gen_extra_template_data, handle_post_generate_command, handle_post_generate_help_text, substitute_variable_in_text,
};
use crate::prelude::*;
use crate::template_engine::TemplateEngine;
use crate::types::{ConfigFileType, TemplateConfig, TemplateData, TemplateDataType};
use crate::utils;
use colored::*;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

#[derive(Debug, PartialEq)]
pub enum ProjectTemplate {
    InBuilt(String),
    Github(String, String),
    Local(String),
    AnyGitRepo(String),
}

impl ProjectTemplate {
    pub fn parse<T: AsRef<str>>(template: T) -> ProjectTemplate {
        let template = template.as_ref().trim();

        if template.starts_with(constants::TEMPLATE_PREFIX_FILE) {
            ProjectTemplate::Local((&template[constants::TEMPLATE_PREFIX_FILE.len()..]).to_owned())
        } else if template.starts_with(constants::TEMPLATE_PREFIX_HTTP)
            || template.starts_with(constants::TEMPLATE_PREFIX_HTTPS)
        {
            ProjectTemplate::AnyGitRepo(template.to_owned())
        } else if template.contains("/") {
            let mut parts = template.split("/");
            ProjectTemplate::Github(parts.next().unwrap().to_owned(), parts.next().unwrap().to_owned())
        } else {
            ProjectTemplate::InBuilt(template.to_owned())
        }
    }

    pub fn disburse<P: AsRef<Path>>(&self, project_dir: P) -> crate::Result<()> {
        match self {
            ProjectTemplate::InBuilt(ref name) => self.disburse_in_built_template(name.as_str(), project_dir),
            ProjectTemplate::Github(ref handle, ref repo) => {
                self.disburse_github_template(handle.as_str(), repo.as_str(), project_dir)
            }
            ProjectTemplate::Local(ref path) => self.disburse_local_template(path.as_str(), project_dir),
            ProjectTemplate::AnyGitRepo(ref url) => self.disburse_any_git_repo_template(url.as_str(), project_dir),
        }
    }

    fn disburse_in_built_template<P: AsRef<Path>>(&self, name: &str, project_dir: P) -> crate::Result<()> {
        let repo_url = Url::parse(constants::TEMPLATE_IN_BUILT_PREFIX)
            .unwrap()
            .join(name)
            .context("Failed to create in-built template repo URL")?;

        let template_path = self.clone_repo(&repo_url, format!("Cloning template: {}", name.green()).as_str())?;

        self.gen_source_code(template_path.as_path(), project_dir.as_ref())?;

        fs::remove_dir_all(template_path.as_path()).wrap()?;

        Ok(())
    }

    fn disburse_github_template<P: AsRef<Path>>(&self, handle: &str, repo: &str, project_dir: P) -> crate::Result<()> {
        let repo_url = Url::parse(constants::TEMPLATE_GITHUB_PREFIX)
            .unwrap()
            .join(format!("{}/", handle).as_str())
            .and_then(|url| url.join(repo))
            .context("Failed to create Github template repo URL")?;

        let template_path = self.clone_repo(
            &repo_url,
            format!("Cloning template: {}", format!("{}/{}", handle, repo).as_str().green()).as_str(),
        )?;

        self.gen_source_code(template_path.as_path(), project_dir.as_ref())?;

        fs::remove_dir_all(template_path.as_path()).wrap()?;

        Ok(())
    }

    fn disburse_local_template<P: AsRef<Path>>(&self, path: &str, project_dir: P) -> crate::Result<()> {
        let template_path = Path::new(path)
            .canonicalize()
            .context("Provided local template doesn't exist")?;

        self.gen_source_code(template_path, project_dir)?;

        Ok(())
    }

    fn disburse_any_git_repo_template<P: AsRef<Path>>(&self, url: &str, project_dir: P) -> crate::Result<()> {
        let repo_url = Url::parse(url).context("Provided template Git repo URL is invalid")?;

        let template_path = self.clone_repo(&repo_url, format!("Cloning template: {}", url.green()).as_str())?;

        self.gen_source_code(template_path.as_path(), project_dir)?;

        fs::remove_dir_all(template_path.as_path()).wrap()?;

        Ok(())
    }

    fn clone_repo(&self, repo_url: &Url, msg: &str) -> crate::Result<PathBuf> {
        let clone_dir = env::temp_dir().join(utils::gen_uuid());

        fs::create_dir_all(clone_dir.as_path()).context(format!(
            "Couldn't create a temporary folder to clone a repo: {}",
            clone_dir.to_str().unwrap_or("")
        ))?;

        println!("{}", msg);

        // The command: git clone <url> --branch <branch> --single-branch [<folder>]
        let output = Command::new("git")
            .arg("clone")
            .arg(repo_url.as_str())
            .args(&["--branch", constants::TEMPLATE_GIT_REPO_BRANCH])
            .arg("--single-branch")
            .arg(clone_dir.as_os_str())
            .output()
            .context("The 'git' command not found")?;

        println!();

        if !output.status.success() {
            return Err(crate::Error::new(format!(
                "The provided template not found: {}",
                String::from_utf8(output.stderr).unwrap_or(String::new())
            )));
        }

        Ok(clone_dir)
    }

    fn gen_source_code<T: AsRef<Path>, P: AsRef<Path>>(&self, template_dir: T, project_dir: P) -> crate::Result<()> {
        let template_dir = template_dir.as_ref();
        let project_dir = project_dir.as_ref();

        let mut template_config = self.extract_template_config(template_dir)?;
        self.resolve_template_config(&mut template_config, template_dir, project_dir)?;

        let template_source_dir = template_dir
            .join(template_config.template.path.as_str())
            .canonicalize()
            .context("Template source not found")?;

        let template_meta = &template_config.template;
        let template_engine = TemplateEngine::parse(template_meta.engine.as_str()).ok_or_else(|| {
            crate::Error::new(format!(
                "Unsupported template engine specified in the provided template: {}",
                template_meta.engine.as_str()
            ))
        })?;

        let template_data =
            data_prompts::ask_data(&template_config).context("Failed to get template data from the user")?;

        let boilerplato_ignore_file_path = template_source_dir.join(constants::TEMPLATE_BOILERPLATO_IGNORE_FILE_NAME);
        let boilerplato_ignore_file_holder = {
            if boilerplato_ignore_file_path.exists() {
                gitignore::File::new(boilerplato_ignore_file_path.as_path()).ok()
            } else {
                None
            }
        };

        let git_ignore_file_path = template_source_dir.join(constants::TEMPLATE_GIT_IGNORE_FILE_NAME);
        let git_ignore_file_holder = {
            if let (ProjectTemplate::Local(_), true) = (self, git_ignore_file_path.exists()) {
                gitignore::File::new(git_ignore_file_path.as_path()).ok()
            } else {
                None
            }
        };

        let ignore_checker = |entry_path: &Path| {
            if let Some(file_name) = entry_path.file_name() {
                if constants::TEMPLATE_IGNORED_FILES.contains(&file_name) {
                    return true;
                }
            }

            if let Some(ref f) = boilerplato_ignore_file_holder {
                if let Some(ignored) = f.is_excluded(entry_path).ok() {
                    if ignored {
                        return true;
                    }
                }
            }

            if let Some(ref f) = git_ignore_file_holder {
                if let Some(ignored) = f.is_excluded(entry_path).ok() {
                    if ignored {
                        return true;
                    }
                }
            }

            return false;
        };

        println!();

        // println!(
        //     "\nCreating a new app in {}\n",
        //     project_dir.to_str().unwrap_or("").green()
        // );

        self.walk_template_dir(template_source_dir.as_path(), &ignore_checker, &|entry_full_path| {
            self.gen_a_single_code_file(
                template_source_dir.as_path(),
                project_dir,
                entry_full_path,
                template_meta.extension.as_str(),
                &template_engine,
                &template_data,
            )?;
            Ok(())
        })?;

        self.initialize_git_to_project_dir(project_dir)?;

        println!();

        if let Some(ref val) = template_config.post_generate {
            handle_post_generate_command(
                val,
                template_dir,
                template_source_dir.as_path(),
                project_dir,
                &template_data,
            )?;
        }

        println!(
            "\nSuccess! Created {} at {}\n",
            project_dir.file_name().and_then(|p| p.to_str()).unwrap_or("").green(),
            project_dir.to_str().unwrap_or("").green()
        );

        if let Some(ref val) = template_config.help_text {
            handle_post_generate_help_text(
                val,
                template_dir,
                template_source_dir.as_path(),
                project_dir,
                &template_data,
                &template_engine,
            )?;
        }

        Ok(())
    }

    fn resolve_template_config(
        &self,
        template_config: &mut TemplateConfig,
        template_dir: &Path,
        project_dir: &Path,
    ) -> crate::Result<()> {
        if template_config.template.path.is_empty() {
            template_config.template.path = constants::TEMPLATE_DEFAULT_TEMPLATE_PATH.to_owned();
        }

        if template_config.template.extension.is_empty() {
            template_config.template.extension = constants::TEMPLATE_DEFAULT_FILE_EXTENSION.to_owned();
        }

        if let Some(app_name) = project_dir.file_name().and_then(|name| name.to_str()) {
            let found = template_config
                .data
                .iter_mut()
                .find(|data| data.name == constants::TEMPLATE_DATA_APP_NAME);

            if let Some(app_name_data) = found {
                app_name_data.data_type = TemplateDataType::String;
                app_name_data.values = None;
                app_name_data.required = false;
                app_name_data.default_value = Some(Value::String(app_name.to_owned()));
            } else {
                template_config.data.insert(
                    0,
                    TemplateData {
                        name: constants::TEMPLATE_DATA_APP_NAME.to_owned(),
                        data_type: TemplateDataType::String,
                        values: None,
                        required: false,
                        default_value: Some(Value::String(app_name.to_owned())),
                        message: format!("Enter app name: "),
                    },
                );
            }
        }

        let extra_data = gen_extra_template_data(
            template_dir,
            template_dir
                .join(template_config.template.path.as_str())
                .canonicalize()
                .context("Template source not found")?
                .as_path(),
            project_dir,
        );
        template_config
            .data
            .iter_mut()
            .filter(|d| d.data_type == TemplateDataType::String && !d.required)
            .for_each(|d| {
                d.default_value
                    .as_ref()
                    .and_then(|val| val.as_str())
                    .map(|s| substitute_variable_in_text(s, &HashMap::new(), &extra_data))
                    .and_then(|val| {
                        d.default_value = Some(Value::String(val));
                        Some(())
                    });
            });

        Ok(())
    }

    fn gen_a_single_code_file(
        &self,
        template_source_dir: &Path,
        project_dir: &Path,
        template_file_full_path: &Path,
        template_file_extension: &str,
        template_engine: &TemplateEngine,
        template_data: &HashMap<&str, Value>,
    ) -> crate::Result<()> {
        let template_file_rel_path = template_file_full_path.strip_prefix(template_source_dir).wrap()?;
        let (is_template_file, template_file_rel_path_without_template_extension) = template_file_rel_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| utils::or(ext == &template_file_extension[1..], Some(()), None))
            .and_then(|_| template_file_rel_path.to_str())
            .map(|rel_path| Path::new(&rel_path[..(rel_path.len() - template_file_extension.len())]))
            .map(|rel_path| (true, rel_path))
            .unwrap_or((false, template_file_rel_path));

        println!(
            "{} {}",
            "Generating".green(),
            template_file_rel_path_without_template_extension.to_str().unwrap_or("")
        );

        let project_actual_file_path = project_dir.join(template_file_rel_path_without_template_extension);
        if let Some(parent) = project_actual_file_path.parent() {
            fs::create_dir_all(parent).context(format!(
                "Couldn't create the missing project dir: {}",
                parent.to_str().unwrap_or("")
            ))?;
        }

        if project_actual_file_path.exists() {
            project_actual_file_path
                .to_str()
                .map(|path| PathBuf::from(format!("{}.old", path)))
                .and_then(|new_path| {
                    fs::rename(project_actual_file_path.as_path(), new_path.as_path())
                        .ok()
                        .map(|_| new_path)
                })
                .and_then(|new_path| {
                    println!(
                        "{} You had a `{}` file, we renamed it to `{}`",
                        "Warning:".yellow(),
                        project_actual_file_path
                            .strip_prefix(project_dir)
                            .ok()
                            .and_then(|rel_path| rel_path.to_str())
                            .unwrap_or(""),
                        new_path
                            .strip_prefix(project_dir)
                            .ok()
                            .and_then(|rel_path| rel_path.to_str())
                            .unwrap_or("")
                    );
                    Some(())
                });
        }

        if !is_template_file {
            fs::copy(template_file_full_path, project_actual_file_path).context(format!(
                "Failed to copy the template file: {}",
                template_file_full_path.to_str().unwrap_or("")
            ))?;
        } else {
            let template_text = fs::read_to_string(template_file_full_path).context(format!(
                "Couldn't read the template file: {}",
                template_file_full_path.to_str().unwrap_or("")
            ))?;

            let actual_code = template_engine
                .render_template(template_text.as_str(), template_data)
                .context(format!(
                    "Failed to generate actual code from template file: {}",
                    template_file_full_path.to_str().unwrap_or("")
                ))?;

            fs::write(project_actual_file_path.as_path(), actual_code.as_bytes()).context(format!(
                "Can't write generated code to project file: {}",
                project_actual_file_path.to_str().unwrap_or("")
            ))?;
        }

        Ok(())
    }

    fn initialize_git_to_project_dir(&self, project_dir: &Path) -> crate::Result<()> {
        let output = Command::new("git")
            .arg("init")
            .current_dir(project_dir)
            .output()
            .context("The 'git' command not found")?;

        if !output.status.success() {
            return Err(crate::Error::new(format!(
                "Failed to initialize git to the project dir: {}",
                String::from_utf8(output.stderr).unwrap_or(String::new())
            )));
        }

        Ok(())
    }

    fn extract_template_config<T: AsRef<Path>>(&self, template_dir: T) -> crate::Result<TemplateConfig> {
        let json_file = constants::TEMPLATE_CONFIG_FILE_JSON;
        let yaml_file = constants::TEMPLATE_CONFIG_FILE_YAML;
        let yml_file = constants::TEMPLATE_CONFIG_FILE_YML;
        let tpl_path = template_dir.as_ref();

        let (file_type, file_name, config_text) = fs::read_to_string(tpl_path.join(json_file))
            .map(|s| (ConfigFileType::JSON, json_file, s))
            .or_else(|_| fs::read_to_string(tpl_path.join(yaml_file)).map(|s| (ConfigFileType::YAML, yaml_file, s)))
            .or_else(|_| fs::read_to_string(tpl_path.join(yml_file)).map(|s| (ConfigFileType::YAML, yml_file, s)))
            .context(format!(
                "No config file found in the provided template: {} or {} or {} file is required",
                json_file, yaml_file, yml_file
            ))?;

        TemplateConfig::parse(config_text.as_str(), file_type)
            .context(format!("Invalid config file in the provided template: {}", file_name))
    }

    fn walk_template_dir<T: AsRef<Path>, F: Fn(&Path) -> crate::Result<()>, I: Fn(&Path) -> bool>(
        &self,
        template_source_dir: T,
        ignore_checker: &I,
        walker: &F,
    ) -> crate::Result<()> {
        let template_source_dir = template_source_dir.as_ref();
        let path_str = template_source_dir.to_str().unwrap_or("");

        for entry in fs::read_dir(template_source_dir).context(format!("Failed to walk template dir: {}", path_str))? {
            let entry = entry.context(format!(
                "Failed to fetch an entry details in template dir: {}",
                path_str
            ))?;
            let entry_full_path = entry.path();

            if ignore_checker(entry_full_path.as_path()) {
                continue;
            }

            let file_type = entry.file_type().context(format!(
                "Failed to fetch entry metadata: {}",
                entry_full_path.to_str().unwrap_or("")
            ))?;

            if file_type.is_dir() {
                self.walk_template_dir(entry_full_path.as_path(), ignore_checker, walker)?;
            } else {
                walker(entry_full_path.as_path())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_template_parse() {
        assert_eq!(
            ProjectTemplate::parse("react-redux-router-nodejs"),
            ProjectTemplate::InBuilt("react-redux-router-nodejs".to_owned())
        );
        assert_eq!(
            ProjectTemplate::parse("john/bar"),
            ProjectTemplate::Github("john".to_owned(), "bar".to_owned()),
        );
        assert_eq!(
            ProjectTemplate::parse("file:../abc"),
            ProjectTemplate::Local("../abc".to_owned()),
        );
        assert_eq!(
            ProjectTemplate::parse("https://github.com/foo/bar.git"),
            ProjectTemplate::AnyGitRepo("https://github.com/foo/bar.git".to_owned()),
        );
        assert_eq!(
            ProjectTemplate::parse("http://github.com/foo/bar.git"),
            ProjectTemplate::AnyGitRepo("http://github.com/foo/bar.git".to_owned()),
        );
    }
}
