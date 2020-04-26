use lazy_static::lazy_static;
use regex::Regex;
use std::ffi::OsStr;

pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
pub const APP_AUTHOR: &'static str = "Rousan Ali <hello@rousan.io> (https://rousan.io)";

pub const TEMPLATE_PREFIX_FILE: &'static str = "file:";
pub const TEMPLATE_PREFIX_HTTPS: &'static str = "https:";
pub const TEMPLATE_PREFIX_HTTP: &'static str = "http:";
pub const TEMPLATE_IN_BUILT_PREFIX: &'static str = "https://github.com/boilerplato/";
pub const TEMPLATE_GITHUB_PREFIX: &'static str = "https://github.com/";
pub const TEMPLATE_GIT_REPO_BRANCH: &'static str = "master";

pub const TEMPLATE_ENGINE_HANDLEBARS: &'static str = "handlebars";

pub const TEMPLATE_CONFIG_FILE_JSON: &'static str = "boilerplato.json";
pub const TEMPLATE_CONFIG_FILE_YAML: &'static str = "boilerplato.yaml";
pub const TEMPLATE_CONFIG_FILE_YML: &'static str = "boilerplato.yml";
pub const TEMPLATE_BOILERPLATO_IGNORE_FILE_NAME: &'static str = ".boilerplatoignore";
pub const TEMPLATE_GIT_IGNORE_FILE_NAME: &'static str = ".gitignore";

pub const TEMPLATE_TYPE_BOOL_POSSIBLE_TRUTHY_INPUTS: [&'static str; 2] = ["yes", "y"];
pub const TEMPLATE_TYPE_BOOL_POSSIBLE_FALSY_INPUTS: [&'static str; 2] = ["no", "n"];

pub const TEMPLATE_TYPE_SEMVER_DEFAULT_VALUE: &'static str = "1.0.0";
pub const TEMPLATE_DEFAULT_TEMPLATE_PATH: &'static str = ".";
pub const TEMPLATE_DEFAULT_FILE_EXTENSION: &'static str = ".boiler";

pub const TEMPLATE_DATA_APP_NAME: &'static str = "appName";

pub const TEMPLATE_OS_FLAG_ALL: &'static str = "all";

pub const TEMPLATE_EXTRA_VAR_APP_NAME: &'static str = "APP_NAME";
pub const TEMPLATE_EXTRA_VAR_PROJECT_DIR_FULL_PATH: &'static str = "APP_FULL_PATH";
pub const TEMPLATE_EXTRA_VAR_PROJECT_DIR_REL_PATH: &'static str = "APP_REL_PATH";
pub const TEMPLATE_EXTRA_VAR_TEMPLATE_PATH: &'static str = "TEMPLATE_PATH";
pub const TEMPLATE_EXTRA_VAR_TEMPLATE_SOURCE_PATH: &'static str = "TEMPLATE_SOURCE_PATH";

pub const SEARCH_REPO_GITHUB_API_ENDPOINT: &'static str = "https://api.github.com/search/repositories";
pub const BOILERPLATO_GITHUB_HANDLE: &'static str = "boilerplato";

lazy_static! {
    pub static ref RE_COMMA_SEPARATOR: Regex = Regex::new(r"[,\s]+").unwrap();
}

lazy_static! {
    pub static ref TEMPLATE_IGNORED_FILES: Vec<&'static OsStr> = vec![
        ".git".as_ref(),
        "node_modules".as_ref(),
        TEMPLATE_CONFIG_FILE_JSON.as_ref(),
        TEMPLATE_CONFIG_FILE_YAML.as_ref(),
        TEMPLATE_CONFIG_FILE_YML.as_ref(),
        TEMPLATE_BOILERPLATO_IGNORE_FILE_NAME.as_ref()
    ];
}
