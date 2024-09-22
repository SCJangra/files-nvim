use std::sync::Arc;

use nvim_oxi::{
	conversion::{Error as ConversionError, FromObject, ToObject},
	lua,
	serde::{Deserializer, Serializer},
	Object,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{traits::LogErr, ExplorerConfig, ExplorerKeymaps};

/// Global configuration of the plugin.
static mut CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
	Arc::new(Config {
		explorer: ExplorerConfig { keymaps: ExplorerKeymaps { quit: String::from("q"), enter: String::from("<CR>") } },
	})
});

/// Configuration of this plugin.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
	pub explorer: ExplorerConfig,
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
		Self::clone().to_object().log_err().ok()
	}

	/// Replace the current configuration by the given [`Object`].
	pub fn set_config(object: Object) {
		let Ok(config) = Self::from_object(object).log_err() else { return };
		unsafe { *CONFIG = Arc::new(config) };
	}
}

impl FromObject for Config {
	fn from_object(object: Object) -> Result<Self, ConversionError> {
		Self::deserialize(Deserializer::new(object)).map_err(Into::into)
	}
}

impl ToObject for Config {
	fn to_object(self) -> Result<Object, ConversionError> {
		self.serialize(Serializer::new()).map_err(Into::into)
	}
}

impl lua::Poppable for Config {
	unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
		let object = Object::pop(lstate)?;
		Self::from_object(object).map_err(lua::Error::pop_error_from_err::<Self, _>)
	}
}

impl lua::Pushable for Config {
	unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
		self.to_object()
			.map_err(lua::Error::push_error_from_err::<Self, _>)?
			.push(lstate)
	}
}
