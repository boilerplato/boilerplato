use crate::constants;
use crate::prelude::*;
use colored::*;
use reqwest::blocking::Client;
use serde_json::Value;

pub fn search_templates_from_registry(search_text: &str) -> crate::Result<()> {
    let results = search_templates_from_github(search_text)?;

    if results.len() == 0 {
        println!();
        println!("   {}", "No templates found!".red());
        println!();
        return Ok(());
    }

    println!();
    for (name, desc) in results {
        println!(
            "   {} {}",
            name.as_str(),
            format!("$ {} my-app -t {}", constants::APP_NAME, name.as_str())
                .as_str()
                .cyan()
        );
        println!("   {}", desc.as_str().bright_black());
        println!();
    }

    Ok(())
}

fn search_templates_from_github(search_text: &str) -> crate::Result<Vec<(String, String)>> {
    let url = format!(
        "{}?q={}+in:name,description+org:{}&sort=stars&order=desc",
        constants::SEARCH_REPO_GITHUB_API_ENDPOINT,
        search_text,
        constants::BOILERPLATO_GITHUB_HANDLE
    );

    let resp = Client::new()
        .get(url.as_str())
        .header(
            "User-Agent",
            format!("{} v{}", constants::APP_NAME, constants::APP_VERSION),
        )
        .send()
        .context("Couldn't connect with Github API server to fetch templates list")?;

    let data = resp.json::<Value>().context("Couldn't parse the Github API response")?;

    Ok(data
        .get("items")
        .and_then(|val| val.as_array())
        .iter()
        .map(|val| val.iter())
        .flatten()
        .filter_map(|val: &Value| {
            val.get("name").and_then(|name| {
                name.as_str().and_then(|name| {
                    val.get("description")
                        .and_then(|desc| desc.as_str())
                        .map(|desc| (name, desc))
                })
            })
        })
        .map(|(name, desc)| (name.to_string(), desc.to_string()))
        .collect())
}
