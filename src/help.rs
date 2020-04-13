use colored::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref TEMPLATE_HELP_TEXT: String = format!(
        "\
The {} can be one of:
    - a template published on https://github.com/boilerplato: e.g. {}
    - a github repo under an username or an organization: e.g. {}
    - a local template path relative to the current working directory: e.g. {}
    - any git repo having boilerplato.yml or boilerplato.json file: e.g. {}\n\n\
        ",
        "--template".cyan(),
        "rust-cli-template".green(),
        "john/web-app-template".green(),
        "file:../my-custom-template".green(),
        "https://github.com/foo/bar.git".green()
    );
}

lazy_static! {
    static ref EXAMPLES_HELP_TEXT: String = format!(
        "\
Examples:
    - create a react app: {}
    - create a Rust CLI app: {}\n\n\
        ",
        "$ boilerplato my-app --template react-redux-router-nodejs".green(),
        "$ boilerplato my-rust-cli-app --template rust-cli-app".green()
    );
}

lazy_static! {
    static ref APP_SHORT_USAGE_TEXT: String = format!("boilerplato {} [OPTIONS]", "<project-directory>".green());
}

pub fn app_help_text() -> String {
    format!("{}{}", &*TEMPLATE_HELP_TEXT, &*EXAMPLES_HELP_TEXT)
}

pub fn app_short_usage_text() -> String {
    format!("{}", &*APP_SHORT_USAGE_TEXT)
}
