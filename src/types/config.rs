use std::sync::Arc;

use nvim_oxi::{
	conversion::{Error as ConversionError, FromObject, ToObject},
	lua,
	serde::{Deserializer, Serializer},
	Object,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static mut CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(Config {}));

/// Configuration of this plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

impl Config {
	/// Get an [`Arc`] to the global config.
	pub fn arc_clone() -> Arc<Self> {
		unsafe { CONFIG.clone() }
	}

	/// Clone the global config.
	pub fn clone() -> Self {
		unsafe { (**CONFIG).clone() }
	}

	/// Get a global Neovim [`Object`] that represents the current configuration. This clones the
	/// global config.
	pub fn get_config(_: ()) -> Option<Object> {
		// TODO: Return error somehow.
		Self::clone().to_object().ok()
	}

	/// Replace the current configuration by the given [`Object`].
	pub fn set_config(object: Object) {
		// TODO: Return error somehow.
		let Ok(config) = Self::from_object(object) else { return };
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
