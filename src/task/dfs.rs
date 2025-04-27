use std::fs;

use rayon::iter::{ParallelBridge, ParallelExtend, ParallelIterator};

use crate::{task::TaskResult, types::File};

pub(crate) struct Dfs {
	stack: Vec<Node>,
}

struct Node {
	file: File,
	visited: bool,
}

impl Dfs {
	pub fn new(files: Vec<File>) -> Self {
		Self { stack: files.into_iter().map(|file| Node { file, visited: false }).collect() }
	}
}

impl Iterator for Dfs {
	type Item = TaskResult<File>;

	fn next(&mut self) -> Option<Self::Item> {
		let node = self.stack.pop()?;

		if node.visited || !node.file.is_dir() {
			return Some(Ok(node.file));
		}

		let read_dir = match fs::read_dir(&node.file.path) {
			Ok(read_dir) => read_dir,
			Err(err) => return Some(Err(err.into())),
		};

		let read_dir = read_dir
			.par_bridge()
			// TODO: Do not ignore errors.
			.filter_map(|d| d.ok())
			// TODO: Do not ignore errors.
			.map(|d| File::from_path(d.path()))
			.filter_map(|res| res.ok())
			.map(|file| Node { file, visited: false });

		self.stack.push(Node { file: node.file, visited: true });
		self.stack.par_extend(read_dir);

		self.next()
	}
}
