use nvim_oxi::libuv::AsyncHandle;
use std::sync::{mpsc::Sender, Arc};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

use crate::{traits::*, types::Result, ArcPath, DIR_CACHE};
use crate::{types::File, ArcFiles};

/// List the files of a directory.
#[derive(derive_more::Debug)]
pub struct List {
	dir: ArcPath,
	#[debug(skip)]
	handler: AsyncHandle,
	sender: Sender<ListResult>,
}

/// Successful response of a [`List`] command.
pub struct ListResponse {
	pub dir: ArcPath,
	pub files: ArcFiles,
}

/// Value returned from a list task.
pub type ListResult = Result<ListResponse>;

impl List {
	pub fn new(dir: ArcPath, handler: AsyncHandle, sender: Sender<ListResult>) -> Self {
		Self { dir, handler, sender }
	}

	async fn list(dir: ArcPath) -> ListResult {
		if let Some(files) = DIR_CACHE.get(&dir) {
			return Ok(ListResponse { dir: Arc::clone(&dir), files: Arc::clone(&files) });
		}

		let read_dir = tokio::fs::read_dir(dir.as_ref()).await?;
		let files = ReadDirStream::new(read_dir);

		let files = files
			.filter_map(|r| r.ok())
			.map(|d| File { path: d.path() })
			.collect::<Vec<_>>()
			.await;

		let files = Arc::new(files);

		DIR_CACHE.insert(Arc::clone(&dir), Arc::clone(&files));

		Ok(ListResponse { dir, files })
	}
}

impl Task for List {
	type Result = Vec<File>;

	async fn execute(self) {
		self.sender.send(Self::list(self.dir).await).ok();
		self.handler.send().ok();
	}
}
