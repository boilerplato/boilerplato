use colored::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref TEMPLATE_HELP_TEXT: String = format!(
        "\
The {} can be one of:
    - a template published on https://github.com/boilerplato: e.g. {}
    - a Github repo under an username or an organization: e.g. {}
    - a local template path relative to the current working directory: e.g. {}
    - any git repo having boilerplato.yml or boilerplato.json file: e.g. {}\n\n\
        ",
        "--template".cyan(),
        "react-nodejs-template".green(),
        "john/web-app-template".green(),
        "file:../my-custom-template".green(),
        "https://github.com/foo/bar.git".green()
    );
}

lazy_static! {
    static ref EXAMPLES_HELP_TEXT: String = format!(
        "\
Examples:
    - create a react.js app: {}
    - create a Rust CLI app: {}\n\n\
        ",
        "$ boilerplato my-app --template react-nodejs-template".green(),
        "$ boilerplato my-app --template rust-cli-template".green()
    );
}

lazy_static! {
    static ref SUB_COMMAND_SEARCH_EXAMPLES_HELP_TEXT: String = format!(
        "\
Examples:
    {}
    {}
        ",
        "$ boilerplato search nodejs".green(),
        "$ boilerplato search rust".green()
    );
}

lazy_static! {
    static ref APP_SHORT_USAGE_TEXT: String = format!("boilerplato {} [OPTIONS]", "<project-directory>".green());
}

lazy_static! {
    static ref SUB_COMMAND_SEARCH_SHORT_USAGE_TEXT: String =
        format!("boilerplato search {} [OPTIONS]", "<search-text>".green());
}

pub fn app_help_text() -> String {
    format!("{}{}", &*TEMPLATE_HELP_TEXT, &*EXAMPLES_HELP_TEXT)
}

pub fn app_short_usage_text() -> String {
    format!("{}", &*APP_SHORT_USAGE_TEXT)
}

pub fn sub_command_search_help_text() -> String {
    format!("{}", &*SUB_COMMAND_SEARCH_EXAMPLES_HELP_TEXT)
}

pub fn sub_command_search_short_usage_text() -> String {
    format!("{}", &*SUB_COMMAND_SEARCH_SHORT_USAGE_TEXT)
}
