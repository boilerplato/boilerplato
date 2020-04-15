use crate::prelude::*;
use crate::template_engine::handlebars::helpers::{concat, json_str, ternary};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use serde::Serialize;

#[macro_use]
mod helpers;

#[derive(Debug)]
pub struct HandlebarsTemplateEngine<'a> {
    handle: Handlebars<'a>,
}

impl<'a> HandlebarsTemplateEngine<'a> {
    pub fn new() -> HandlebarsTemplateEngine<'a> {
        let mut h = Handlebars::new();

        helper!(h, "json_str", json_str);
        helper!(h, "concat", concat);
        helper!(h, "ternary", ternary);

        HandlebarsTemplateEngine { handle: h }
    }

    pub fn render_template<S: AsRef<str>, D: Serialize>(&self, template_text: S, data: &D) -> crate::Result<String> {
        let template_text = template_text.as_ref();
        self.handle
            .render_template(template_text, data)
            .context("Failed to parse the template file as handlebars")
    }
}
