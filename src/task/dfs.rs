use std::fs;

use rayon::iter::{ParallelBridge, ParallelExtend, ParallelIterator};

use crate::types::File;

use super::TaskResult;

pub(crate) struct Dfs {
	stack: Vec<File>,
}

impl Dfs {
	pub fn new(files: Vec<File>) -> Self {
		Self { stack: files }
	}
}

impl Iterator for Dfs {
	type Item = TaskResult<File>;

	fn next(&mut self) -> Option<Self::Item> {
		let file = self.stack.pop()?;

		if !file.is_dir() {
			return Some(Ok(file));
		}

		let read_dir = match fs::read_dir(&file.path) {
			Ok(read_dir) => read_dir,
			Err(err) => return Some(Err(err.into())),
		};

		let read_dir = read_dir
			.par_bridge()
			.filter_map(|d| d.ok())
			.map(|d| File::from_path(d.path()))
			.filter_map(|res| res.ok());

		self.stack.par_extend(read_dir);

		Some(Ok(file))
	}
}
