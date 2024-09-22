use crate::LogErr;
use nvim_oxi::{api::Error as ApiError, conversion::Error as ConversionError};

impl<T> LogErr for Result<T, ConversionError> {
	fn log_err(self) -> Self {
		if let Err(ref err) = self {
			nvim_oxi::print!("Error: {err}");
		}

		self
	}
}

impl<T> LogErr for Result<T, ApiError> {
	fn log_err(self) -> Self {
		if let Err(ref err) = self {
			nvim_oxi::print!("Error: {err}");
		}

		self
	}
}
