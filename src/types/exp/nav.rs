use std::path::PathBuf;

pub struct Navigator {
	dirs: Vec<PathBuf>,
	index: usize,
	max_index: usize,
}
#[derive(Clone)]
pub enum Nav {
	Next,
	Prev,
	New,
	Noop,
}

impl Navigator {
	pub fn new(current_dir: PathBuf) -> Self {
		Self { dirs: vec![current_dir], index: 0, max_index: 0 }
	}

	/// Get the previous directory.
	pub fn next(&self) -> Option<&PathBuf> {
		let new_index = self.index.checked_add(1)?;

		if new_index > self.max_index {
			return None;
		}

		self.dirs.get(new_index)
	}

	/// Get the next directory.
	pub fn prev(&self) -> Option<&PathBuf> {
		let new_index = self.index.checked_sub(1)?;
		self.dirs.get(new_index)
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

	/// Insert a new directory.
	pub fn insert(&mut self, dir: PathBuf) {
		self.index = self.index.saturating_add(1);
		self.max_index = self.index;

		match self.index >= self.dirs.len() {
			true => self.dirs.push(dir),
			false => self.dirs[self.index] = dir,
		}
	}
}
