use rhai::EvalAltResult;

pub mod actions;
pub mod mail_context;
pub mod types;

pub type EngineResult<T> = Result<T, Box<EvalAltResult>>;