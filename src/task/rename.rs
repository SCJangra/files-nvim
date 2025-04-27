use std::{
	path::PathBuf,
	sync::atomic::{self, AtomicBool},
};

use crate::{
	task::TaskResult,
	traits::{AtomicTask, TaskHandle},
};

use nvim_oxi as nvim;

pub(crate) struct Rename {
	/// Indicates whether the rename operation has been canceled.
	canceled: AtomicBool,
	/// The file to rename.
	file: PathBuf,
	/// The new name for the file.
	new_name: String,
}

impl Rename {
	pub fn new(file: PathBuf, new_name: String) -> Self {
		Self { file, new_name, canceled: AtomicBool::new(false) }
	}
}

impl AtomicTask for Rename {
	type Response = ();

	fn execute(&self) -> TaskResult<Self::Response> {
		let from = self.file.as_path();
		let mut to = self.file.clone();
		to.set_file_name(&self.new_name);

		std::fs::rename(from, to)?;

		Ok(())
	}
}

impl TaskHandle for Rename {
	fn cancel(&self) {
		self.canceled.store(true, atomic::Ordering::Release);
	}

	fn is_cancelled(&self) -> bool {
		self.canceled.load(atomic::Ordering::Acquire)
	}

	#[inline(always)]
	fn is_unique(&self) -> bool {
		false
	}

	fn progress(&self, _width: u32) -> Vec<nvim::String> {
		vec![]
	}
}
