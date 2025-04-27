use std::{
	fmt,
	sync::atomic::Ordering,
	time::{Duration, Instant},
};

use crate::{
	task::TaskResult,
	types::{Progress, Result},
	utils::fun,
};

use nvim_oxi as nvim;

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

	/// Get the current progress of this task.
	fn progress(&self, width: u32) -> Vec<nvim::String>;
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

pub trait WriteExt: fmt::Write {
	fn write_size(&mut self, size: f64, unit: &str) -> fmt::Result {
		match size {
			0.0..10.0 => self.write_fmt(format_args!("{size:>3.1}{unit}")),
			_ => self.write_fmt(format_args!("{size:>3.0}{unit}")),
		}
	}

	fn write_prog_size(&mut self, prog: &Progress, sep: &str) -> fmt::Result {
		let acquire = Ordering::Acquire;

		let done = prog.done.load(acquire);
		let total = prog.total.load(acquire);

		let (done, done_unit) = fun::bytes_to_size(done);
		let (total, total_unit) = fun::bytes_to_size(total);

		self.write_size(done, done_unit)?;
		self.write_str(sep)?;
		self.write_size(total, total_unit)
	}

	#[allow(unused)]
	fn write_prog_count(&mut self, prog: &Progress, sep: &str) -> fmt::Result {
		let acquire = Ordering::Acquire;

		let done = prog.done.load(acquire);
		let total = prog.total.load(acquire);

		self.write_fmt(format_args!("{done}{sep}{total}"))
	}

	fn write_duration(&mut self, seconds: u64) -> fmt::Result {
		let hours = seconds / 3600;
		let minutes = (seconds % 3600) / 60;
		let seconds = seconds % 60;

		self.write_fmt(format_args!("{hours:02}:{minutes:02}:{seconds:02}"))
	}
}
