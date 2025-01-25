#[macro_export]
macro_rules! lua_interop {
	($type:ty) => {
		impl nvim_oxi::conversion::FromObject for $type {
			fn from_object(object: nvim_oxi::Object) -> Result<Self, nvim_oxi::conversion::Error> {
				use nvim_oxi::serde::Deserializer;
				use serde::Deserialize;

				Self::deserialize(Deserializer::new(object)).map_err(Into::into)
			}
		}

		impl nvim_oxi::conversion::ToObject for $type {
			fn to_object(self) -> Result<nvim_oxi::Object, nvim_oxi::conversion::Error> {
				use nvim_oxi::serde::Serializer;
				use serde::Serialize;

				self.serialize(Serializer::new()).map_err(Into::into)
			}
		}

		impl nvim_oxi::lua::Poppable for $type {
			unsafe fn pop(lstate: *mut nvim_oxi::lua::ffi::State) -> Result<Self, nvim_oxi::lua::Error> {
				use nvim_oxi::{conversion::FromObject, Object};

				let object = Object::pop(lstate)?;
				Self::from_object(object).map_err(nvim_oxi::lua::Error::pop_error_from_err::<Self, _>)
			}
		}

		impl nvim_oxi::lua::Pushable for $type {
			unsafe fn push(
				self,
				lstate: *mut nvim_oxi::lua::ffi::State,
			) -> Result<std::ffi::c_int, nvim_oxi::lua::Error> {
				use nvim_oxi::conversion::ToObject;

				self.to_object()
					.map_err(nvim_oxi::lua::Error::push_error_from_err::<Self, _>)?
					.push(lstate)
			}
		}
	};
}
