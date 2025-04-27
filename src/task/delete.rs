use std::{
	fmt::Write,
	fs,
	sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	time::Instant,
};

use crate::{
	error::TaskError,
	task::{dfs::Dfs, TaskResult},
	traits::{Task, TaskHandle, WriteExt},
	types::{File, Progress},
	utils::fun,
};

use nvim_oxi as nvim;

pub struct Delete {
	files: Vec<File>,
	canceled: AtomicBool,
	progress: Arc<DeleteProgress>,
}

pub struct DeleteProgress {
	inner: Progress,
	duration: AtomicU64,
}

impl Delete {
	pub fn new(files: Vec<File>) -> Self {
		Self {
			files,
			canceled: AtomicBool::new(false),
			progress: Arc::new(DeleteProgress { inner: Progress::default(), duration: AtomicU64::new(0) }),
		}
	}
}

impl Task for Delete {
	type Update = ();

	fn execute(&self) -> super::TaskResult<impl Iterator<Item = TaskResult<Self::Update>>> {
		let start_time = Instant::now();

		let (sender, receiver) = crossbeam_channel::unbounded();

		let files = self.files.clone();
		let progress = Arc::clone(&self.progress);

		rayon::spawn(move || {
			Dfs::new(files).filter_map(|file| file.ok()).for_each(|file| {
				progress.inner.total.fetch_add(1, Ordering::Release);
				sender.send(file).ok();
			});
		});

		let progress = Arc::clone(&self.progress);

		let iter = receiver
			.into_iter()
			.map(|file| match file.is_dir() {
				true => fs::remove_dir(&file.path).map_err(TaskError::from),
				false => fs::remove_file(&file.path).map_err(TaskError::from),
			})
			.inspect(move |res| {
				progress.duration.store(start_time.elapsed().as_secs(), Ordering::Release);
				if res.is_ok() {
					progress.inner.done.fetch_add(1, Ordering::Release);
				}
			});

		Ok(iter)
	}
}

impl TaskHandle for Delete {
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

	fn progress(&self, width: u32) -> Vec<nvim::String> {
		let mut header = String::with_capacity(width as usize);

		header.write_str("Delete ").ok();
		header.write_char('[').ok();
		header.write_prog_count(&self.progress.inner, "/").ok();
		header.write_char(']').ok();
		header.write_str(" files").ok();

		let duration = {
			let mut d = String::new();
			d.write_duration(self.progress.duration.load(Ordering::Acquire)).ok();
			d
		};

		let header_width = width.saturating_sub(duration.len() as u32 - 1) as usize;

		let (header, dots) = fun::trim_str(&header, header_width);

		vec![nvim::string!("{header}{dots:<w$}{duration}", w = header_width - header.len() + dots.len())]
	}
}
