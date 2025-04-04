use serde::{Deserialize, Serialize};

use crate::types::Field;

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
	pub keymaps: ExplorerKeymaps,
	pub fields: Vec<Field>,
	pub name_width: usize,
	pub column_seperator: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerKeymaps {
	pub quit: String,
	pub enter: String,
	pub next: String,
	pub prev: String,
	pub up: String,
	pub rename: String,
	pub copy: String,
	pub cut: String,
	pub paste: String,
	pub create: String,
	pub delete: String,
	/// Show task manager in current window.
	pub tm_current: String,
}

crate::lua_interop!(ExplorerConfig);
