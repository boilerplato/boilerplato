extern crate boilerplato;
use boilerplato::constants;
use clap::{App, Arg, SubCommand};
use colored::*;

fn main() {
    let matches = App::new(constants::APP_NAME)
        .version(constants::APP_VERSION)
        .version_short("v")
        .author(constants::APP_AUTHOR)
        .about("A powerful tool to generate boilerplate source code from a template.\nPlease visit https://boilerplato.com for more information.")
        .arg(
            Arg::with_name("projectDirectory")
                .help("The project directory to create source files into")
                .value_name("project-directory")
                .index(1)
                .required(false),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .value_name("path-to-template")
                .help("Specify a template for the new project")
                .takes_value(true)
                .required(false),
        )
        .usage(boilerplato::help::app_short_usage_text().as_str())
        .after_help(boilerplato::help::app_help_text().as_str())
        .subcommand(
            SubCommand::with_name("search")
                .version(constants::APP_VERSION)
                .version_short("v")
                .author(constants::APP_AUTHOR)
                .about("Search templates published on https://github.com/boilerplato.")
                .arg(Arg::with_name("query")
                    .help("The search text to query templates")
                    .value_name("search-text")
                    .index(1)
                    .required(false))
                .usage(boilerplato::help::sub_command_search_short_usage_text().as_str())
                .after_help(boilerplato::help::sub_command_search_help_text().as_str())
        )
        .get_matches();

    if let Some(project_directory) = matches.value_of("projectDirectory") {
        if let Some(template) = matches.value_of("template") {
            if let Err(err) = boilerplato::generator::gen_source_code_from_template(project_directory, template) {
                eprintln!("{} {}", "error:".red(), err)
            }
        } else {
            eprintln!(
                "{} The following required arguments were not provided:\n{}\n\nFor more information try {}",
                "error:".red(),
                "    --template <path-to-template>".red(),
                "boilerplato --help".green()
            )
        }
    } else if let Some(search_matches) = matches.subcommand_matches("search") {
        if let Some(search_query) = search_matches.value_of("query") {
            if let Err(err) = boilerplato::search::search_templates_from_registry(search_query) {
                eprintln!("{} {}", "error:".red(), err)
            }
        } else {
            eprintln!(
                "{} The following required arguments were not provided:\n{}\n\nFor more information try {}",
                "error:".red(),
                "    <search-text>".red(),
                "boilerplato search --help".green()
            )
        }
    } else {
        eprintln!(
            "{} The required arguments were not provided.\n\nFor more information try {}",
            "error:".red(),
            "boilerplato --help".green()
        )
    }
}
