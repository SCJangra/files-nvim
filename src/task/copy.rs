use std::{
	fs,
	io::{BufReader, BufWriter, Read, Write},
	path::PathBuf,
	ptr,
	sync::atomic::{AtomicBool, AtomicPtr, Ordering},
	time::Duration,
};

use crate::traits::{Task, TaskHandle};

use super::TaskResult;

pub struct Copy {
	canceled: AtomicBool,
	file: PathBuf,
	dest: PathBuf,
	progress: AtomicPtr<Progress>,
	update_interval: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
	size: u64,
	done: u64,
}

pub struct Copier {
	file: PathBuf,
	dest: PathBuf,
	buf: Vec<u8>,
	reader: BufReader<fs::File>,
	writer: BufWriter<fs::File>,
}

impl Copier {
	pub fn new(file: PathBuf, mut dest: PathBuf) -> TaskResult<Self> {
		dest.push(file.file_name().unwrap_or_default());

		let from = fs::File::open(file.as_path())?;
		let to = fs::OpenOptions::new().write(true).truncate(true).open(dest.as_path())?;

		let reader = BufReader::new(from);
		let writer = BufWriter::new(to);

		Ok(Self { file, dest, buf: Vec::with_capacity(5_000_000), reader, writer })
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
	pub fn new(file: PathBuf, dest: PathBuf) -> Self {
		Self {
			file,
			dest,
			canceled: AtomicBool::new(false),
			progress: AtomicPtr::new(ptr::null_mut()),
			update_interval: Duration::from_millis(500),
		}
	}
}

impl Task for Copy {
	type Progress = TaskResult<Progress>;

	fn execute(&self) -> TaskResult<impl Iterator<Item = Self::Progress>> {
		let meta = fs::metadata(self.file.as_path())?;

		let mut prog = Progress { size: meta.len(), done: 0 };

		let iter = Copier::new(self.file.clone(), self.dest.clone())?
			.take_while(|_| !self.is_cancelled())
			.map(move |res| {
				let bytes = res?;

				prog.done += bytes as u64;

				self.progress.store(&mut prog, Ordering::Release);

				Ok(prog)
			});

		Ok(iter)
	}

	fn update_interval(&self) -> Duration {
		self.update_interval
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
