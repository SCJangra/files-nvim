use std::path::PathBuf;

use nvim_oxi::api;
use serde::{Deserialize, Serialize};

use crate::{error::Error, lua_interop, types::Result};

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
#[serde(rename_all = "snake_case")]
pub enum Field {
	Size,
}

lua_interop!(Field);

impl File {
	pub(crate) fn open(&self) -> Result<()> {
		let path = self.path.as_path();

		let mime = tree_magic_mini::from_filepath(path).ok_or_else(|| Error::UnknownFile)?;

		if mime.starts_with("text/") {
			api::command(format!("edit {}", path.to_str().unwrap_or_default()).as_str())?
		} else {
			open::that_in_background(path);
		};

		Ok(())
	}
}
