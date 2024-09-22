use nvim_oxi::{
	conversion::{Error as ConversionError, FromObject, ToObject},
	lua,
	serde::{Deserializer, Serializer},
	Object,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
	pub keymaps: ExplorerKeymaps,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplorerKeymaps {
	pub quit: String,
	pub enter: String,
}

impl FromObject for ExplorerKeymaps {
	fn from_object(object: Object) -> Result<Self, ConversionError> {
		Self::deserialize(Deserializer::new(object)).map_err(Into::into)
	}
}

impl ToObject for ExplorerKeymaps {
	fn to_object(self) -> Result<Object, ConversionError> {
		self.serialize(Serializer::new()).map_err(Into::into)
	}
}

impl lua::Poppable for ExplorerKeymaps {
	unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
		let object = Object::pop(lstate)?;
		Self::from_object(object).map_err(lua::Error::pop_error_from_err::<Self, _>)
	}
}

impl lua::Pushable for ExplorerKeymaps {
	unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
		self.to_object()
			.map_err(lua::Error::push_error_from_err::<Self, _>)?
			.push(lstate)
	}
}

impl FromObject for ExplorerConfig {
	fn from_object(object: Object) -> Result<Self, ConversionError> {
		Self::deserialize(Deserializer::new(object)).map_err(Into::into)
	}
}

impl ToObject for ExplorerConfig {
	fn to_object(self) -> Result<Object, ConversionError> {
		self.serialize(Serializer::new()).map_err(Into::into)
	}
}

impl lua::Poppable for ExplorerConfig {
	unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
		let object = Object::pop(lstate)?;
		Self::from_object(object).map_err(lua::Error::pop_error_from_err::<Self, _>)
	}
}

impl lua::Pushable for ExplorerConfig {
	unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
		self.to_object()
			.map_err(lua::Error::push_error_from_err::<Self, _>)?
			.push(lstate)
	}
}
