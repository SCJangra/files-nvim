use nvim_oxi::Function;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputOpts {
	pub prompt: String,
	pub default: String,
}

pub type InputCallback = Function<Option<String>, ()>;

pub type Input = Function<(InputOpts, InputCallback), ()>;

crate::impl_pushable!(InputOpts);
