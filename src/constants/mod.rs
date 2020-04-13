use lazy_static::lazy_static;
use regex::Regex;
use std::ffi::OsStr;

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

pub const TEMPLATE_TYPE_BOOL_POSSIBLE_TRUTHY_INPUTS: [&'static str; 2] = ["yes", "y"];
pub const TEMPLATE_TYPE_BOOL_POSSIBLE_FALSY_INPUTS: [&'static str; 2] = ["no", "n"];

pub const TEMPLATE_TYPE_SEMVER_DEFAULT_VALUE: &'static str = "1.0.0";

lazy_static! {
    pub static ref RE_COMMA_SEPARATOR: Regex = Regex::new(r"[,\s]+").unwrap();
}

lazy_static! {
    pub static ref TEMPLATE_IGNORED_FILES: Vec<&'static OsStr> = vec![
        ".git".as_ref(),
        TEMPLATE_CONFIG_FILE_JSON.as_ref(),
        TEMPLATE_CONFIG_FILE_YAML.as_ref(),
        TEMPLATE_CONFIG_FILE_YML.as_ref(),
        TEMPLATE_BOILERPLATO_IGNORE_FILE_NAME.as_ref()
    ];
}
