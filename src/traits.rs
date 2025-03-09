use rayon::iter::ParallelIterator;

use crate::types::Result;

pub trait LogErr {
	fn log_err(&self);

	fn log_error(self) -> Self;
}

pub trait WithModifiable {
	fn with_modifiable(&self, f: impl FnOnce() -> Result<()>) -> Result<()>;
}

pub trait Task {
	type Progress;

	/// Execute this task.
	fn execute(&self) -> impl ParallelIterator<Item = Self::Progress>;
}

pub trait AtomicTask {
	/// Response type of this task.
	type Response;

	/// Execute this task.
	fn execute(&self) -> Self::Response;
}

pub trait TaskHandle {
	/// Cancel this task.
	fn cancel(&self);

	/// Is this task canceled?
	fn is_cancelled(&self) -> bool;

	/// Whether a single instance of this task should exist at a time. For tasks that are `unique`,
	/// starting a new instance cancels the previous one.
	fn is_unique(&self) -> bool;
}
