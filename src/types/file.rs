use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use nvim_oxi::api;
use serde::{Deserialize, Serialize};

use crate::{error::Error, lua_interop, task::TaskResult, types::Result};

/// A file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
	pub path: PathBuf,
	pub ty: FileType,
	pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
	File,
	DirectoryEmpty,
	DirectoryFull,
	Symlink,
	Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

	pub(crate) fn is_dir(&self) -> bool {
		matches!(self.ty, FileType::DirectoryEmpty | FileType::DirectoryFull)
	}

	pub(crate) fn name(&self) -> Option<&OsStr> {
		self.path.file_name()
	}

	pub(crate) fn name_str(&self) -> Option<&str> {
		self.name().and_then(|n| n.to_str())
	}

	pub(crate) fn from_path(path: PathBuf) -> TaskResult<Self> {
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

		Ok(Self { path, ty, size: meta.len() })
	}

	// TODO: take a `Path` instead of `&str`
	pub(crate) fn touch_new(path: &str, mut dest: PathBuf) -> TaskResult<(PathBuf, fs::File)> {
		let path = path.trim();
		let parts = path.split_terminator('/').collect::<Vec<_>>();
		// SAFETY: `parts` cannot be empty here, so this won't panic.
		let last = parts.len() - 1;
		let create_dir = path.ends_with("/");

		let mut index = 0;
		let file = loop {
			let name = parts[index];

			dest.push(name);

			match (index == last, create_dir) {
				(true, true) | (false, _) => {
					let res = fs::create_dir(dest.as_path());

					match res {
						Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
						Err(err) => Err(err),
						Ok(v) => Ok(v),
					}?
				},
				(true, false) => break fs::File::create_new(dest.as_path())?,
			};

			index += 1
		};

		Ok((dest, file))
	}

	// TODO: take a `Path` instead of `&str`
	pub(crate) fn create_new(path: &str, dest: PathBuf) -> TaskResult<Self> {
		let (path, _) = Self::touch_new(path, dest)?;
		let file = Self::from_path(path)?;
		Ok(file)
	}
}

impl Ord for File {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.path.cmp(&other.path)
	}
}

impl PartialOrd for File {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
