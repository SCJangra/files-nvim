use nvim_oxi as nvim;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputOpts {
	pub prompt: String,
	pub default: String,
}

pub type Input = nvim::Function<(InputOpts, nvim::Function<Option<String>, ()>), ()>;

crate::impl_pushable!(InputOpts);
