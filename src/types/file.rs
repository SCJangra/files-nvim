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

	pub(crate) fn create_dir(path: &str, mut dest: PathBuf) -> TaskResult<PathBuf> {
		let path = path.trim_start_matches('/').trim_end_matches('/');

		dest.push(path);
		let res = fs::create_dir_all(&dest);

		match res {
			Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(dest),
			Err(err) => Err(err.into()),
			Ok(_) => Ok(dest),
		}
	}

	pub(crate) fn create_file(path: &str, mut dest: PathBuf) -> TaskResult<(PathBuf, fs::File)> {
		let path = path.trim_start_matches('/').trim_end_matches('/');
		let (dir_path, file_name) = path.rsplit_once('/').unwrap_or(("", path));

		if !dir_path.is_empty() {
			dest = Self::create_dir(dir_path, dest)?;
		}

		dest.push(file_name);

		let file = fs::File::create_new(&dest)?;

		Ok((dest, file))
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

#[cfg(test)]
mod tests {
	use assertables::*;
	use tempfile::tempdir;

	use super::*;

	#[test]
	fn create_dir() {
		let root = assert_ok!(tempdir());
		let root = root.path();

		let path_bad = "///a/b////c///";
		let path_good = "a/b/c";

		let path = assert_ok!(File::create_dir(path_bad, root.to_path_buf()));

		assert_eq!(path, root.join(path_good));
		assert!(path.exists());
		assert!(path.is_dir());
	}

	#[test]
	fn create_file() {
		let root = assert_ok!(tempdir());
		let root = root.path();

		let path_bad = "///a/b////c///";
		let path_good = "a/b/c";

		let (path, _) = assert_ok!(File::create_file(path_bad, root.to_path_buf()));

		assert_eq!(path, root.join(path_good));
		assert!(path.exists());
		assert!(path.is_file());
	}
}
