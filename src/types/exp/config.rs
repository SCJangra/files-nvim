use serde::{Deserialize, Serialize};

use crate::types::Field;

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
	pub keymaps: ExplorerKeymaps,
	pub fields: Vec<Field>,
	pub name_width: usize,
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
