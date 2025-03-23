use std::{
	collections::VecDeque,
	path::{Path, PathBuf},
};

pub struct Navigator {
	dirs: VecDeque<PathBuf>,
	index: usize,
	max_index: usize,
}
#[derive(Clone, Copy)]
pub enum Nav {
	Next,
	Prev,
	New,
	Up,
	Noop,
}

impl Navigator {
	pub fn new(current_dir: PathBuf) -> Self {
		Self { dirs: vec![current_dir].into(), index: 0, max_index: 0 }
	}

	/// Get the previous directory.
	pub fn next(&self) -> Option<&Path> {
		let new_index = self.index.checked_add(1)?;

		if new_index > self.max_index {
			return None;
		}

		self.dirs.get(new_index).map(|d| d.as_path())
	}

	/// Get the next directory.
	pub fn prev(&self) -> Option<&Path> {
		let new_index = self.index.checked_sub(1)?;
		self.dirs.get(new_index).map(|d| d.as_path())
	}

	/// Get the parent directory.
	pub fn up(&self) -> Option<&Path> {
		self.dirs.get(self.index).and_then(|d| d.parent())
	}

	pub fn current_dir(&self) -> &Path {
		// SAFETY: Current directory should always exist.
		&self.dirs[self.index]
	}

	/// Go to the previous directory.
	pub fn go_to_next(&mut self) {
		let new_index = self.index.saturating_add(1);

		if new_index > self.max_index {
			return;
		}

		self.index = new_index;
	}

	/// Go to the next directory.
	pub fn go_to_prev(&mut self) {
		self.index = self.index.saturating_sub(1);
	}

	/// Go to the parent directory.
	pub fn got_to_up(&mut self) {
		let Some(parent) = self.dirs.get(self.index).and_then(|d| d.parent()) else { return };

		if self.prev() == Some(parent) {
			self.go_to_prev();
			return;
		}

		self.dirs.push_front(parent.to_path_buf());
		self.max_index = self.max_index.saturating_add(1);
	}

	/// Insert a new directory.
	pub fn insert(&mut self, dir: PathBuf) {
		self.index = self.index.saturating_add(1);
		self.max_index = self.index;

		match self.index >= self.dirs.len() {
			true => self.dirs.push_back(dir),
			false => self.dirs[self.index] = dir,
		}
	}
}
