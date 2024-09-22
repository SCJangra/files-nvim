use crate::LogErr;
use nvim_oxi::conversion::Error;

impl<T> LogErr for Result<T, Error> {
	fn log_err(self) -> Self {
		if let Err(ref err) = self {
			nvim_oxi::print!("Error: {err}");
		}

		self
	}
}
