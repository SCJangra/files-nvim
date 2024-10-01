use crate::types::Result;

pub trait LogErr {
	fn log_err(&self);

	fn log_error(self) -> Self;
}

pub trait WithModifiable {
	fn with_modifiable(&self, f: impl FnOnce() -> Result<()>) -> Result<()>;
}
