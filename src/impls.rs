use crate::{
	error::Error,
	traits::{LogErr, WithModifiable},
};

use nvim_oxi::{
	self as nvim,
	api::{Buffer, Error as ApiError},
	conversion::Error as ConversionError,
};

impl<T> LogErr for Result<T, ConversionError> {
	fn log_err(&self) {
		if let Err(ref err) = self {
			nvim::print!("Error: {err}");
		}
	}

	fn log_error(self) -> Self {
		self.log_err();
		self
	}
}

impl<T> LogErr for Result<T, ApiError> {
	fn log_err(&self) {
		if let Err(ref err) = self {
			nvim::print!("Error: {err}");
		}
	}

	fn log_error(self) -> Self {
		self.log_err();
		self
	}
}

impl Drop for Error {
	fn drop(&mut self) {
		self.log_err();
	}
}

impl WithModifiable for Buffer {
	fn with_modifiable(&self, f: impl FnOnce() -> crate::Result<()>) -> crate::Result<()> {
		self.set_option("ma", true)?;
		f()?;
		self.set_option("ma", false)?;
		Ok(())
	}
}
