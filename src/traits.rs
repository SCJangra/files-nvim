pub trait Task {
	type Result;

	async fn execute(self);
}

pub trait LogErr {
	fn log_err(self) -> Self;
}
