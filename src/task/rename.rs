use std::{
	path::PathBuf,
	sync::atomic::{self, AtomicBool},
};

use crate::{
	task::TaskResult,
	traits::{AtomicTask, TaskHandle},
};

pub(crate) struct Rename {
	/// Indicates whether the rename operation has been canceled.
	canceled: AtomicBool,
	/// Index of the file in the explorer.
	file_index: usize,
	/// The file to rename.
	file: PathBuf,
	/// The new name for the file.
	new_name: String,
}

pub(crate) struct RenameResponse {
	/// Index of the file in the explorer.
	pub file_index: usize,
	/// The new name for the file.
	pub new_name: String,
}

impl Rename {
	pub fn new(file_index: usize, file: PathBuf, new_name: String) -> Self {
		Self { file_index, file, new_name, canceled: AtomicBool::new(false) }
	}
}

impl AtomicTask for Rename {
	type Response = RenameResponse;

	fn execute(&self) -> TaskResult<Self::Response> {
		let from = self.file.as_path();
		let mut to = self.file.clone();
		to.set_file_name(&self.new_name);

		std::fs::rename(from, to)?;

		Ok(RenameResponse { file_index: self.file_index, new_name: self.new_name.clone() })
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
}
