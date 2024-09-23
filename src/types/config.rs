use std::sync::Arc;

use nvim_oxi::{
	conversion::{FromObject, ToObject},
	Object,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{traits::LogErr, ExplorerConfig, ExplorerKeymaps, Icons};

/// Global configuration of the plugin.
static mut CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
	Arc::new(Config {
		explorer: ExplorerConfig { keymaps: ExplorerKeymaps { quit: String::from("q"), enter: String::from("<CR>") } },
		icons: Icons { file_name: Default::default(), extension: Default::default() },
	})
});

/// Configuration of this plugin.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
	pub explorer: ExplorerConfig,
	pub icons: Icons,
}

impl Config {
	/// Get an [`Arc`] to the global config.
	pub fn arc_clone() -> Arc<Self> {
		unsafe { Arc::clone(&*CONFIG) }
	}

	/// Clone the global config.
	pub fn clone() -> Self {
		unsafe { (**CONFIG).clone() }
	}

	/// Get a global Neovim [`Object`] that represents the current configuration. This clones the
	/// global config.
	pub fn get_config(_: ()) -> Option<Object> {
		Self::clone().to_object().log_error().ok()
	}

	/// Replace the current configuration by the given [`Object`].
	pub fn set_config(object: Object) {
		let Ok(config) = Self::from_object(object).log_error() else { return };
		unsafe { *CONFIG = Arc::new(config) };
	}
}

crate::lua_interop!(Config);
