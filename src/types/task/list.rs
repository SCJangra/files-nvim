use nvim_oxi::libuv::AsyncHandle;
use std::{path::PathBuf, sync::mpsc::Sender};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{traits, types::*};

/// List the files of a directory.
pub struct List {
	dir: PathBuf,
	handler: AsyncHandle,
	sender: Sender<ListResult>,
}

/// Successful response of a [`List`] command.
pub struct ListResponse {
	pub dir: PathBuf,
	pub files: Vec<File>,
}

/// Value returned from a list task.
pub type ListResult = Result<ListResponse>;

impl List {
	pub fn new(dir: PathBuf, handler: AsyncHandle, sender: Sender<ListResult>) -> Self {
		Self { dir, handler, sender }
	}

	async fn list(dir: PathBuf) -> ListResult {
		let read_dir = tokio::fs::read_dir(&dir).await?;
		let files = ReadDirStream::new(read_dir);

		let files = files
			.filter_map(|r| r.ok())
			.then(|d| async move {
				let path = d.path();
				let meta = tokio::fs::metadata(&path).await?;

				let ty = if meta.is_file() {
					FileType::File
				} else if meta.is_dir() {
					let child = tokio::fs::read_dir(&path).await?.next_entry().await?;

					match child {
						Some(_) => FileType::DirectoryFull,
						None => FileType::DirectoryEmpty,
					}
				} else if meta.is_symlink() {
					FileType::Symlink
				} else {
					FileType::Unknown
				};

				Ok::<_, tokio::io::Error>(File { path, ty })
			})
			.filter_map(|res| res.ok())
			.collect::<Vec<_>>()
			.await;

		Ok(ListResponse { dir, files })
	}
}

impl traits::Task for List {
	type Result = Vec<File>;

	async fn execute(self) {
		self.sender.send(Self::list(self.dir).await).ok();
		self.handler.send().ok();
	}
}
