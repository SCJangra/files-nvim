use std::fmt;

use crate::{error::*, traits::*, types};

use nvim_oxi::{
	self as nvim,
	api::{self, opts::OptionOpts, Buffer, Error as ApiError},
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
	fn with_modifiable(&self, f: impl FnOnce() -> types::Result<()>) -> types::Result<()> {
		let opts = OptionOpts::builder().buffer(*self).build();

		api::set_option_value("ma", true, &opts)?;
		f()?;
		api::set_option_value("ma", false, &opts)?;

		Ok(())
	}
}

impl<I: Iterator> IterExt for I {}

impl<T: fmt::Write> WriteExt for T {}
