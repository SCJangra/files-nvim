use std::time::{Duration, Instant};

use crate::{task::TaskResult, types::Result};

pub trait LogErr {
	fn log_err(&self);

	fn log_error(self) -> Self;
}

pub trait WithModifiable {
	fn with_modifiable(&self, f: impl FnOnce() -> Result<()>) -> Result<()>;
}

pub trait Task {
	type Update;

	/// Execute this task.
	fn execute(&self) -> TaskResult<impl Iterator<Item = TaskResult<Self::Update>>>;
}

pub trait AtomicTask {
	/// Response type of this task.
	type Response;

	/// Execute this task.
	fn execute(&self) -> TaskResult<Self::Response>;
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

pub trait IterExt: Iterator + Sized {
	fn for_each_interval<F>(self, interval: Duration, mut func: F)
	where
		F: FnMut(Self::Item),
	{
		let mut time = Instant::now();

		let mut iter = self.peekable();

		loop {
			let Some(item) = iter.next() else { break };

			if time.elapsed() < interval && iter.peek().is_some() {
				continue;
			}

			func(item);
			time = Instant::now();
		}
	}
}
