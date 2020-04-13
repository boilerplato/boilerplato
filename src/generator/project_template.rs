use crate::constants;
use crate::data_prompts;
use crate::prelude::*;
use crate::template_engine::TemplateEngine;
use crate::types::{ConfigFileType, TemplateConfig};
use crate::utils;
use colored::*;
use std::env;
use std::fs;
use std::fs::DirEntry;
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

        let template_config = self.extract_template_config(template_dir)?;

        let template_engine = TemplateEngine::parse(template_config.template_engine.as_str()).ok_or_else(|| {
            crate::Error::new(format!(
                "Unsupported template engine specified in the provided template: {}",
                template_config.template_engine
            ))
        })?;

        let template_data =
            data_prompts::ask_data(&template_config).context("Failed to get template data from the user")?;

        let ignore_file_path = template_dir.join(constants::TEMPLATE_BOILERPLATO_IGNORE_FILE_NAME);
        let ignore_file_holder = {
            if ignore_file_path.exists() {
                gitignore::File::new(ignore_file_path.as_path()).ok()
            } else {
                None
            }
        };

        let ignore_checker = |entry_path: &Path| {
            let file_name = entry_path.file_name();
            if let Some(file_name) = file_name {
                if constants::TEMPLATE_IGNORED_FILES.contains(&file_name) {
                    return true;
                }
            }

            if let Some(ref f) = ignore_file_holder {
                if let Some(ignored) = f.is_excluded(entry_path).ok() {
                    if ignored {
                        return true;
                    }
                }
            }

            return false;
        };

        self.walk_template_dir(template_dir, &ignore_checker, &|entry_full_path| {
            let entry_rel_path = entry_full_path.strip_prefix(template_dir).wrap()?;

            println!("{:?}", entry_rel_path);

            Ok(())
        })?;

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
        template_dir: T,
        ignore_checker: &I,
        walker: &F,
    ) -> crate::Result<()> {
        let template_dir = template_dir.as_ref();
        let path_str = template_dir.to_str().unwrap_or("");

        for entry in fs::read_dir(template_dir).context(format!("Failed to walk the template dir: {}", path_str))? {
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
    fn test_template_engine_parse() {
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
