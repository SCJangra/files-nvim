use std::{
	fs,
	io::{BufReader, BufWriter, Read, Write},
	path::PathBuf,
	sync::{
		atomic::{AtomicBool, AtomicPtr, Ordering},
		Arc,
	},
};
use unwrap_or::*;

use crate::{
	error::TaskError,
	traits::{Task, TaskHandle},
	types::{File, Progress},
};

use super::{dfs::Dfs, TaskResult};

pub struct Copy {
	files: Vec<File>,
	dest: PathBuf,
	canceled: AtomicBool,
	progress: Arc<CopyProgress>,
}

pub struct CopyProgress {
	files: Progress,
	bytes: Progress,
	current_file: AtomicPtr<String>,
	current: Progress,
}

pub struct Copier {
	buf: Vec<u8>,
	reader: BufReader<fs::File>,
	writer: BufWriter<fs::File>,
}

impl Copier {
	pub fn new(from: fs::File, to: fs::File) -> Self {
		let reader = BufReader::new(from);
		let writer = BufWriter::new(to);

		Self { buf: vec![0; 5_000_000], reader, writer }
	}
}

impl Iterator for Copier {
	type Item = TaskResult<usize>;

	fn next(&mut self) -> Option<Self::Item> {
		let res = self
			.reader
			.read(&mut self.buf[..])
			.and_then(|bytes| self.writer.write_all(&self.buf[..bytes]).map(|_| bytes));

		match res {
			Ok(0) => None,
			Ok(bytes) => Some(Ok(bytes)),
			Err(err) => Some(Err(err.into())),
		}
	}
}

impl Copy {
	pub fn new(files: Vec<File>, dest: PathBuf) -> Self {
		Self {
			files,
			dest,
			canceled: AtomicBool::new(false),
			progress: Arc::new(CopyProgress {
				files: Progress::default(),
				bytes: Progress::default(),
				current_file: AtomicPtr::new(&mut String::new()),
				current: Progress::default(),
			}),
		}
	}
}

impl Task for Copy {
	type Update = ();

	fn execute(&self) -> TaskResult<impl Iterator<Item = TaskResult<()>>> {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let progress = Arc::clone(&self.progress);
		let files = self.files.clone();

		rayon::spawn(move || {
			Dfs::new(files).filter_map(|file| file.ok()).for_each(|file| {
				progress.files.total.fetch_add(1, Ordering::Release);
				progress.bytes.total.fetch_add(file.size, Ordering::Release);
			});
		});

		let progress = Arc::clone(&self.progress);
		let files = self.files.clone();
		let dest = self.dest.clone();

		let copier = |prefix: &PathBuf, file, dest| {
			let file: File = file?;
			let copy_path = file.path.strip_prefix(prefix.as_path())?;
			let name = copy_path
				.file_name()
				.and_then(|name| name.to_str())
				.ok_or(TaskError::NotUtf8FileNme)?
				.to_string();
			let size = file.size;
			let copy_path = copy_path.to_str().ok_or(TaskError::NotUtf8Path)?;
			let from = fs::File::open(&file.path)?;
			let (file, to) = File::create_file(copy_path, dest)?;

			Ok((file, name, size, Copier::new(from, to)))
		};

		let copy_file = move |prefix, file| {
			for file in Dfs::new(vec![file]).filter(|file| file.as_ref().map(|file| !file.is_dir()).unwrap_or(true)) {
				let copier = copier(&prefix, file, dest.clone());
				let (_, mut name, size, copier) = unwrap_ok_or!(copier, err, {
					sender.send(Err(err)).ok();
					continue;
				});

				progress.current_file.store(&mut name, Ordering::Release);
				progress.current.total.store(size, Ordering::Release);
				progress.current.done.store(0, Ordering::Release);

				for bytes in copier {
					let bytes = unwrap_ok_or!(bytes, err, {
						sender.send(Err(err)).ok();
						continue;
					});

					progress.current.done.fetch_add(bytes as u64, Ordering::Release);

					sender.send(Ok(())).ok();
				}
			}
		};

		rayon::spawn(move || {
			files.into_iter().for_each(|file| {
				let prefix = file.path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
				copy_file(prefix, file);
			})
		});

		Ok(receiver.into_iter())
	}
}

impl TaskHandle for Copy {
	#[inline(always)]
	fn cancel(&self) {
		self.canceled.store(true, Ordering::Release);
	}

	#[inline(always)]
	fn is_cancelled(&self) -> bool {
		self.canceled.load(Ordering::Acquire)
	}

	#[inline(always)]
	fn is_unique(&self) -> bool {
		false
	}
}
