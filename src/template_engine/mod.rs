use crate::constants;
use crate::template_engine::handlebars::HandlebarsTemplateEngine;
use serde::Serialize;

mod handlebars;

#[derive(Debug)]
pub enum TemplateEngine<'a> {
    Handlebars(HandlebarsTemplateEngine<'a>),
}

impl<'a> TemplateEngine<'a> {
    pub fn parse<E: AsRef<str>>(template_engine_id: E) -> Option<TemplateEngine<'a>> {
        let template_engine_id = template_engine_id.as_ref().trim().to_lowercase();

        if template_engine_id == constants::TEMPLATE_ENGINE_HANDLEBARS {
            Some(TemplateEngine::Handlebars(HandlebarsTemplateEngine::new()))
        } else {
            None
        }
    }

    pub fn render_template<S: AsRef<str>, D: Serialize>(&self, template_text: S, data: &D) -> crate::Result<String> {
        match self {
            TemplateEngine::Handlebars(ref engine) => engine.render_template(template_text, data),
        }
    }
}
