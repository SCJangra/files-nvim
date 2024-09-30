use std::{
	fs, io,
	sync::atomic::{self, AtomicBool},
};

use crossbeam_channel::Receiver;
use nvim_oxi::libuv::AsyncHandle;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use std::{path::PathBuf, sync::mpsc::Sender};

use crate::{
	error::Error,
	types::{File, FileType, Result},
};

/// List the files of a directory.
pub struct List {
	dir: PathBuf,
	handler: AsyncHandle,
	sender: Sender<ListResult>,
}

/// Successful response of a [`List`] command.
pub struct ListResponse {
	pub files: Vec<File>,
}

/// Value returned from a list task.
pub type ListResult = Result<ListResponse>;

impl List {
	pub fn new(dir: PathBuf, handler: AsyncHandle, sender: Sender<ListResult>) -> Self {
		Self { dir, handler, sender }
	}

	pub fn exec(self, r: &Receiver<Self>) {
		self.sender.send(Self::do_list(self.dir, r)).ok();
		self.handler.send().ok();
	}

	fn do_list(dir: PathBuf, r: &Receiver<Self>) -> ListResult {
		let read_dir = fs::read_dir(&dir)?;

		let cancelled = AtomicBool::new(false);

		let (mut dirs, mut files): (Vec<_>, Vec<_>) = read_dir
			.par_bridge()
			.take_any_while(|_| match r.is_empty() {
				true => true,
				false => {
					cancelled.store(true, atomic::Ordering::Release);
					false
				},
			})
			.filter_map(|d| d.ok())
			.map(|d| {
				let path = d.path();
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

				Ok::<_, io::Error>(File { path, ty })
			})
			.filter_map(|res| res.ok())
			.partition(|f| matches!(f.ty, FileType::DirectoryFull | FileType::DirectoryEmpty));

		rayon::join(
			|| dirs.par_sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name())),
			|| files.par_sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name())),
		);

		let mut all = Vec::with_capacity(dirs.len() + files.len());
		all.extend_from_slice(&dirs);
		all.extend_from_slice(&files);

		match cancelled.load(atomic::Ordering::Acquire) {
			true => Err(Error::Cancelled),
			false => Ok(ListResponse { files: all }),
		}
	}
}
