use std::{
	fs, io,
	path::PathBuf,
	sync::atomic::{self, AtomicBool},
};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
	error::TaskError,
	traits::{AtomicTask, TaskHandle},
	types::{File, FileType},
};

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
	type Response = Result<Vec<File>, TaskError>;

	fn execute(&self) -> Self::Response {
		if self.is_cancelled() {
			return Err(TaskError::Cancelled);
		}

		let read_dir = fs::read_dir(&self.dir)?;

		let files: Vec<_> = read_dir
			.par_bridge()
			.take_any_while(|_| !self.is_cancelled())
			.filter_map(|d| d.ok())
			.map(|d| {
				let path = d.path();
				let meta = fs::metadata(&path)?;

				let ty = if meta.is_file() {
					FileType::File
				} else if meta.is_dir() {
					let child = fs::read_dir(&path)?.next();

					match child {
						Some(_) => FileType::DirectoryFull,
						None => FileType::DirectoryEmpty,
					}
				} else if meta.is_symlink() {
					FileType::Symlink
				} else {
					FileType::Unknown
				};

				Ok::<_, io::Error>(File { path, ty, size: meta.len() })
			})
			.filter_map(|res| res.ok())
			.collect();

		match self.is_cancelled() {
			true => Err(TaskError::Cancelled),
			false => Ok(files),
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
}
