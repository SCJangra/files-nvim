use std::path::PathBuf;

/// A file.
#[derive(Debug, Clone)]
pub struct File {
	pub path: PathBuf,
	pub ty: FileType,
}

#[derive(Debug, Clone)]
pub enum FileType {
	File,
	DirectoryEmpty,
	DirectoryFull,
	Symlink,
	Unknown,
}
