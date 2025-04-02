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
	pub fn peek_next(&self) -> Option<&Path> {
		let new_index = self.index.checked_add(1)?;

		if new_index > self.max_index {
			return None;
		}

		self.dirs.get(new_index).map(|d| d.as_path())
	}

	/// Get the next directory.
	pub fn peek_prev(&self) -> Option<&Path> {
		let new_index = self.index.checked_sub(1)?;
		self.dirs.get(new_index).map(|d| d.as_path())
	}

	/// Get the parent directory.
	pub fn peek_parent(&self) -> Option<&Path> {
		self.dirs.get(self.index).and_then(|d| d.parent())
	}

	pub fn current_dir(&self) -> &Path {
		// SAFETY: Current directory should always exist.
		&self.dirs[self.index]
	}

	/// Go to the previous directory.
	pub fn next(&mut self) -> Option<&Path> {
		let new_index = self.index.checked_add(1)?;

		if new_index > self.max_index {
			return None;
		}

		let dir = self.dirs.get(new_index).map(|d| d.as_path())?;

		if !dir.try_exists().ok()? {
			return None;
		}

		// This update *MUST* be at the end of the function.
		self.index = new_index;
		Some(dir)
	}

	/// Go to the next directory.
	pub fn prev(&mut self) -> Option<&Path> {
		let new_index = self.index.checked_sub(1)?;

		let dir = self.dirs.get(new_index).map(|d| d.as_path())?;

		if !dir.try_exists().ok()? {
			return None;
		}

		// This update *MUST* be at the end of the function.
		self.index = new_index;
		Some(dir)
	}

	/// Go to the parent directory.
	pub fn up(&mut self) -> Option<&Path> {
		let parent = self.dirs.get(self.index).and_then(|d| d.parent())?;

		if self.peek_prev() == Some(parent) {
			return self.prev();
		}

		self.dirs.push_front(parent.to_path_buf());
		let dir = self.dirs.front().map(|d| d.as_path())?;

		if !dir.try_exists().ok()? {
			return None;
		}

		// This update *MUST* be at the end of the function.
		self.max_index = self.max_index.checked_add(1)?;
		Some(dir)
	}

	/// Insert a new directory.
	pub fn insert(&mut self, dir: PathBuf) -> Option<&Path> {
		let new_index = self.index.checked_add(1)?;

		if !dir.try_exists().ok()? {
			return None;
		}

		match new_index >= self.dirs.len() {
			true => self.dirs.push_back(dir),
			false => self.dirs[new_index] = dir,
		}

		let dir = self.dirs.get(new_index).map(|d| d.as_path())?;

		// These updates *MUST* be at the end of the function.
		self.index = self.index.checked_add(1)?;
		self.max_index = self.index;

		Some(dir)
	}
}
