use crate::constants;
use crate::data_prompts;
use crate::generator::post_generator::{
    gen_extra_template_data, handle_post_generate_command, handle_post_generate_help_text, substitute_variable_in_text,
};
use crate::prelude::*;
use crate::template_engine::TemplateEngine;
use crate::types::{CondFileMap, ConfigFileType, TemplateConfig, TemplateData, TemplateDataType};
use crate::utils;
use colored::*;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::FileType;
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

        let ignore_checker = |entry_full_path: &Path| {
            if let Some(file_name) = entry_full_path.file_name() {
                if constants::TEMPLATE_IGNORED_FILES.contains(&file_name) {
                    return Ok(true);
                }
            }

            if let Some(ref f) = boilerplato_ignore_file_holder {
                if let Some(ignored) = f.is_excluded(entry_full_path).ok() {
                    if ignored {
                        return Ok(true);
                    }
                }
            }

            if let Some(ref f) = git_ignore_file_holder {
                if let Some(ignored) = f.is_excluded(entry_full_path).ok() {
                    if ignored {
                        return Ok(true);
                    }
                }
            }

            if let Some(ref files_map) = template_config.files_map {
                if let Ok(entry_rel_path) = entry_full_path.strip_prefix(template_source_dir.as_path()) {
                    let config = entry_rel_path.to_str().and_then(|s| files_map.get(&s.to_string()));

                    if let Some(config) = config {
                        let check = template_engine
                            .render_template(config.check.as_str(), &template_data)
                            .context(format!(
                                "Couldn't generate check condition for key in 'files' attribute: {}",
                                entry_rel_path.to_str().unwrap_or("")
                            ))?;

                        let check = dbg!(check.trim().to_lowercase());
                        if check.is_empty() || check == "false" {
                            return Ok(true);
                        }
                    }
                }
            }

            if entry_full_path.is_file() {
                let template_versioned_of_file_exists = entry_full_path
                    .to_str()
                    .map(|p| format!("{}{}", p, template_meta.extension))
                    .map(|p| PathBuf::from(p))
                    .map(|p| p.exists());

                if let Some(exists) = template_versioned_of_file_exists {
                    if exists {
                        return Ok(true);
                    }
                }
            }

            return Ok(false);
        };

        println!();

        // println!(
        //     "\nCreating a new app in {}\n",
        //     project_dir.to_str().unwrap_or("").green()
        // );

        self.walk_template_dir(
            template_source_dir.as_path(),
            &ignore_checker,
            &|entry_full_path, rel_path_in_project_dir| {
                self.gen_a_single_code_file(
                    template_source_dir.as_path(),
                    project_dir,
                    entry_full_path,
                    rel_path_in_project_dir,
                    template_meta.extension.as_str(),
                    &template_engine,
                    &template_data,
                )?;
                Ok(())
            },
            &|entry_rel_path, entry_file_name, file_type| {
                if let Some(ref files_map) = template_config.files_map {
                    if let Some(rel_path_str) = entry_rel_path.to_str() {
                        let new_name_template = files_map.get(rel_path_str).and_then(|cond| cond.new_name.as_ref());

                        if let Some(new_name_template) = new_name_template {
                            return template_engine
                                .render_template(new_name_template.as_str(), &template_data)
                                .context(format!(
                                    "Couldn't generate new name for key in 'files' attribute: {}",
                                    rel_path_str
                                ));
                        }
                    }
                }

                if file_type.is_dir() {
                    return Ok(entry_file_name.to_string());
                }

                let template_file_extension = template_meta.extension.as_str();
                let entry_file_name_without_template_extension = entry_rel_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|ext| utils::or(ext == &template_file_extension[1..], Some(()), None))
                    .map(|_| &entry_file_name[..(entry_file_name.len() - template_file_extension.len())])
                    .unwrap_or(entry_file_name);

                Ok(entry_file_name_without_template_extension.to_string())
            },
            template_source_dir.as_path(),
            Path::new(""),
        )?;

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
            println!();
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

        let template_source_dir = template_dir
            .join(template_config.template.path.as_str())
            .canonicalize()
            .context("Template source not found")?;

        let extra_data = gen_extra_template_data(template_dir, template_source_dir.as_path(), project_dir);
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

        if let Some(ref files) = template_config.files {
            let mut files_map = HashMap::with_capacity(files.len());

            for (key, val) in files.iter() {
                let val = match val {
                    Value::String(s) => CondFileMap {
                        check: s.clone(),
                        new_name: None,
                    },
                    Value::Object(m) => CondFileMap {
                        check: m
                            .get(&"check".to_owned())
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .ok_or_else(|| {
                                crate::Error::new(format!(
                                    "The 'files' doesn't have handlebars formatted 'check' condition for: {}",
                                    key
                                ))
                            })?,
                        new_name: m.get("newName").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    },
                    _ => {
                        return Err(crate::Error::new(
                            "The 'files' allows only string or a map as condition",
                        ))
                    }
                };

                let files_path_key_full_path = template_source_dir
                    .join(key)
                    .canonicalize()
                    .context(format!("A path key in 'files' attribute does not exist: {}", key))?;

                let resolved_files_path_key_rel_path = files_path_key_full_path
                    .strip_prefix(template_source_dir.as_path())
                    .wrap()
                    .and_then(|p| {
                        p.to_str().ok_or_else(|| {
                            crate::Error::new(
                                "A path key relative path in 'files' attribute couldn't be converted into a string",
                            )
                        })
                    })
                    .map(|s| s.to_string())
                    .context(format!(
                        "Failed to retrieve relative path for a key in 'files' attribute: {}",
                        key
                    ))?;

                files_map.insert(resolved_files_path_key_rel_path, val);
            }

            template_config.files_map = Some(files_map);
        }

        Ok(())
    }

    fn gen_a_single_code_file(
        &self,
        _template_source_dir: &Path,
        project_dir: &Path,
        template_file_full_path: &Path,
        template_file_rel_path_in_project_dir: &Path,
        template_file_extension: &str,
        template_engine: &TemplateEngine,
        template_data: &HashMap<&str, Value>,
    ) -> crate::Result<()> {
        let is_template_file = template_file_full_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| utils::or(ext == &template_file_extension[1..], Some(()), None))
            .is_some();

        println!(
            "{} {}",
            "Generating".green(),
            template_file_rel_path_in_project_dir.to_str().unwrap_or("")
        );

        let project_actual_file_path = project_dir.join(template_file_rel_path_in_project_dir);
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
            fs::copy(template_file_full_path, project_actual_file_path.as_path()).context(format!(
                "Failed to copy the template file: {} to project file: {}",
                template_file_full_path.to_str().unwrap_or(""),
                project_actual_file_path.to_str().unwrap_or("")
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

    fn walk_template_dir<
        T: AsRef<Path>,
        F: Fn(&Path, &Path) -> crate::Result<()>,
        I: Fn(&Path) -> crate::Result<bool>,
        N: Fn(&Path, &str, &FileType) -> crate::Result<String>,
    >(
        &self,
        template_source_curr_walking_dir: T,
        ignore_checker: &I,
        walker: &F,
        name_generator: &N,
        template_source_dir: &Path,
        rel_path_in_project_dir: &Path,
    ) -> crate::Result<()> {
        let template_source_curr_walking_dir = template_source_curr_walking_dir.as_ref();
        let path_str = template_source_curr_walking_dir.to_str().unwrap_or("");

        for entry in fs::read_dir(template_source_curr_walking_dir)
            .context(format!("Failed to walk template dir: {}", path_str))?
        {
            let entry = entry.context(format!(
                "Failed to fetch an entry details in template dir: {}",
                path_str
            ))?;

            let entry_full_path = entry.path();
            let entry_full_path_str = entry_full_path.to_str().unwrap_or("");

            if ignore_checker(entry_full_path.as_path())? {
                continue;
            }

            let file_type = entry
                .file_type()
                .context(format!("Failed to fetch entry metadata: {}", entry_full_path_str))?;

            let entry_rel_path = entry_full_path.strip_prefix(template_source_dir).context(format!(
                "Couldn't get rel path for a template file: {}",
                entry_full_path_str
            ))?;

            let entry_file_name = entry_full_path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                crate::Error::new(format!(
                    "Couldn't get file name for a template file: {}",
                    entry_full_path_str
                ))
            })?;

            let entry_new_name = name_generator(entry_rel_path, entry_file_name, &file_type)?;
            let rel_path_in_project_dir = rel_path_in_project_dir.join(entry_new_name);

            println!(
                "out: {:?} {:?} {:?}",
                entry_rel_path, entry_file_name, rel_path_in_project_dir
            );

            if file_type.is_dir() {
                self.walk_template_dir(
                    entry_full_path.as_path(),
                    ignore_checker,
                    walker,
                    name_generator,
                    template_source_dir,
                    rel_path_in_project_dir.as_path(),
                )?;
            } else {
                walker(entry_full_path.as_path(), rel_path_in_project_dir.as_path())?;
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
