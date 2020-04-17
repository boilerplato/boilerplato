extern crate boilerplato;
use clap::{App, Arg};
use colored::*;

fn main() {
    let matches = App::new("boilerplato")
        .version(env!("CARGO_PKG_VERSION"))
        .version_short("v")
        .author("Rousan Ali <hello@rousan.io> (https://rousan.io)")
        .about("A powerful tool to generate boilerplate source code from a template.\nPlease visit https://boilerplato.com for more information.")
        .arg(
            Arg::with_name("projectDirectory")
                .help("The project directory to create source files into")
                .value_name("project-directory")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .value_name("path-to-template")
                .help("Specify a template for the new project")
                .takes_value(true)
                .required(true),
        )
        .usage(boilerplato::help::app_short_usage_text().as_str())
        .after_help(boilerplato::help::app_help_text().as_str())
        .get_matches();

    let project_directory = matches.value_of("projectDirectory").unwrap();
    let template = matches.value_of("template").unwrap();

    if let Err(err) = boilerplato::generator::gen_source_code_from_template(project_directory, template) {
        eprintln!("{} {}", "error:".red(), err)
    }
}
