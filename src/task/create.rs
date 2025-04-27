use std::{
	path::PathBuf,
	sync::atomic::{AtomicBool, Ordering},
};

use crate::{
	traits::{AtomicTask, TaskHandle},
	types::File,
};

use super::TaskResult;

use nvim_oxi as nvim;

pub struct Create {
	canceled: AtomicBool,
	path: String,
	dest: PathBuf,
}

impl Create {
	pub(crate) fn new(path: String, dest: PathBuf) -> Self {
		Self { path, dest, canceled: AtomicBool::new(false) }
	}
}

impl AtomicTask for Create {
	type Response = ();

	fn execute(&self) -> TaskResult<Self::Response> {
		match self.path.ends_with('/') {
			true => File::create_dir(&self.path, self.dest.clone()).map(|_| ()),
			false => File::create_file(&self.path, self.dest.clone()).map(|_| ()),
		}
	}
}

impl TaskHandle for Create {
	#[inline(always)]
	fn cancel(&self) {
		self.canceled.store(true, Ordering::Release);
	}

	#[inline(always)]
	fn is_cancelled(&self) -> bool {
		self.canceled.load(Ordering::Acquire)
	}

	#[inline(always)]
	fn is_unique(&self) -> bool {
		false
	}

	fn progress(&self, _width: u32) -> Vec<nvim::String> {
		vec![]
	}
}
