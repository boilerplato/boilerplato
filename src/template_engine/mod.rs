use crate::constants;
use crate::template_engine::handlebars::HandlebarsTemplateEngine;
use serde::Serialize;

mod handlebars;

#[derive(Debug)]
pub enum TemplateEngine {
    Handlebars(HandlebarsTemplateEngine),
}

impl TemplateEngine {
    pub fn parse<E: AsRef<str>>(template_engine: E) -> Option<TemplateEngine> {
        let template_engine = template_engine.as_ref().trim().to_lowercase();
        if template_engine == constants::TEMPLATE_ENGINE_HANDLEBARS {
            Some(TemplateEngine::Handlebars(HandlebarsTemplateEngine {}))
        } else {
            None
        }
    }

    pub fn render_template<S: AsRef<str>, D: Serialize>(template_text: S, data: &D) -> String {
        todo!()
    }
}
