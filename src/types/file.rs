use std::{path::PathBuf, sync::Arc};

/// A file.
#[derive(Debug)]
pub struct File {
	pub path: ArcPath,
	pub ty: FileType,
}

#[derive(Debug)]
pub enum FileType {
	File,
	Directory,
	Symlink,
	Unknown,
}

/// Reference to a vector of files.
pub type ArcFiles = Arc<Vec<File>>;

/// Path to a file.
pub type ArcPath = Arc<PathBuf>;
