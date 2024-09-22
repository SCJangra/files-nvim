mod config;
mod exp;
mod file;
mod task;

pub use config::*;
pub use exp::*;
pub use file::*;
pub use task::*;

pub type Result<T> = std::result::Result<T, crate::error::Error>;
