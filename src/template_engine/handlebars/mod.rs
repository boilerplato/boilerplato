use crate::prelude::*;
use handlebars::Handlebars;
use serde::Serialize;

#[derive(Debug)]
pub struct HandlebarsTemplateEngine<'a> {
    handle: Handlebars<'a>,
}

impl<'a> HandlebarsTemplateEngine<'a> {
    pub fn new() -> HandlebarsTemplateEngine<'a> {
        let handle = Handlebars::new();

        // register helper functions here

        HandlebarsTemplateEngine { handle }
    }

    pub fn render_template<S: AsRef<str>, D: Serialize>(&self, template_text: S, data: &D) -> crate::Result<String> {
        let template_text = template_text.as_ref();
        self.handle
            .render_template(template_text, data)
            .context("Failed to parse the template file as handlebars")
    }
}
