use std::{
	fs,
	path::PathBuf,
	sync::atomic::{AtomicBool, Ordering},
};

use crate::{
	error::TaskError,
	traits::{AtomicTask, TaskHandle},
	types::File,
};

use super::TaskResult;

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
	type Response = TaskResult<File>;

	fn execute(&self) -> Self::Response {
		let path = self.path.trim();
		let parts = path.split_terminator('/').collect::<Vec<_>>();
		// SAFETY: `parts` cannot be empty here, so this won't panic.
		let last = parts.len() - 1;
		let create_dir = path.ends_with("/");

		let mut dest = self.dest.clone();

		parts.iter().enumerate().try_for_each(|(index, name)| {
			dest.push(name);

			match (index == last, create_dir) {
				(true, true) | (false, _) => fs::create_dir(dest.as_path()),
				(true, false) => fs::File::create_new(dest.as_path()).map(|_| ()),
			}
		})?;

		let path = {
			let mut file = self.dest.clone();
			let name = parts[0]; // SAFETY: `parts` cannot be empty here, so this won't panic.
			file.push(name);
			file
		};

		let file = File::from_path(path)?;

		Ok(file)
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
}
