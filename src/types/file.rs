use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::lua_interop;

/// A file.
#[derive(Clone)]
pub struct File {
	pub path: PathBuf,
	pub ty: FileType,
	pub size: u64,
}

#[derive(Clone, Copy)]
pub enum FileType {
	File,
	DirectoryEmpty,
	DirectoryFull,
	Symlink,
	Unknown,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Field {
	Name,
	Size,
}

lua_interop!(Field);
