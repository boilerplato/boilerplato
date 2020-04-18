pub use self::error::{Error, ErrorExt, ResultExt};

pub mod constants;
pub mod data_prompts;
mod error;
pub mod generator;
pub mod help;
pub mod prelude;
pub mod search;
pub mod template_engine;
pub mod types;
pub mod utils;

pub type Result<T> = std::result::Result<T, Error>;
