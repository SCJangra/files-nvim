use std::{
	fs,
	path::PathBuf,
	sync::atomic::{self, AtomicBool},
};

use rayon::iter::{ParallelBridge, ParallelIterator};

use nvim_oxi as nvim;

use crate::{
	error::TaskError,
	traits::{AtomicTask, TaskHandle},
	types::File,
};

use super::TaskResult;

pub(crate) struct List {
	/// The directory to list.
	dir: PathBuf,
	/// Whether this task is canceled or not.
	canceled: AtomicBool,
}

impl List {
	pub fn new(dir: PathBuf) -> Self {
		Self { dir, canceled: AtomicBool::new(false) }
	}
}

impl AtomicTask for List {
	type Response = (PathBuf, Vec<File>);

	fn execute(&self) -> TaskResult<Self::Response> {
		if self.is_cancelled() {
			return Err(TaskError::Cancelled);
		}

		let read_dir = fs::read_dir(&self.dir)?;

		let files: Vec<_> = read_dir
			.par_bridge()
			.take_any_while(|_| !self.is_cancelled())
			.filter_map(|d| d.ok())
			.map(|d| File::from_path(d.path()))
			.filter_map(|res| res.ok())
			.collect();

		match self.is_cancelled() {
			true => Err(TaskError::Cancelled),
			false => Ok((self.dir.clone(), files)),
		}
	}
}

impl TaskHandle for List {
	fn cancel(&self) {
		self.canceled.store(true, atomic::Ordering::Release);
	}

	fn is_cancelled(&self) -> bool {
		self.canceled.load(atomic::Ordering::Acquire)
	}

	#[inline(always)]
	fn is_unique(&self) -> bool {
		true
	}

	fn progress(&self, _width: u32) -> Vec<nvim::String> {
		vec![]
	}
}
