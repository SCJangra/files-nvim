pub trait LogErr {
	fn log_err(&self);

	fn log_error(self) -> Self;
}
