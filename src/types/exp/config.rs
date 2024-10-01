use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
	pub keymaps: ExplorerKeymaps,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerKeymaps {
	pub quit: String,
	pub enter: String,
	pub next: String,
	pub prev: String,
	pub up: String,
}

crate::lua_interop!(ExplorerConfig);
