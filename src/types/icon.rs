use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Icon {
	pub name: String,
	pub icon: char,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Icons {
	pub file_name: HashMap<String, Icon>,
	pub extension: HashMap<String, Icon>,
	pub default: Icon,
	pub dir_empty: Icon,
	pub dir_full: Icon,
}

crate::lua_interop!(Icon);
crate::lua_interop!(Icons);
