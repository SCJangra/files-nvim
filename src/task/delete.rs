use std::{
	fs,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use crate::{
	error::TaskError,
	task::{dfs::Dfs, TaskResult},
	traits::{Task, TaskHandle},
	types::{File, Progress},
};

pub struct Delete {
	files: Vec<File>,
	canceled: AtomicBool,
	progress_interval: Duration,
	progress: Arc<Progress>,
}

impl Delete {
	pub fn new(files: Vec<File>) -> Self {
		Self {
			files,
			canceled: AtomicBool::new(false),
			progress_interval: Duration::from_millis(500),
			progress: Arc::new(Progress::default()),
		}
	}
}

impl Task for Delete {
	type Update = ();

	fn execute(&self) -> super::TaskResult<impl Iterator<Item = TaskResult<Self::Update>>> {
		let (sender, receiver) = crossbeam_channel::unbounded();

		let files = self.files.clone();
		let progress = Arc::clone(&self.progress);

		rayon::spawn(move || {
			Dfs::new(files).filter_map(|file| file.ok()).for_each(|file| {
				progress.total.fetch_add(1, Ordering::Release);
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
				if res.is_ok() {
					progress.done.fetch_add(1, Ordering::Release);
				}
			});

		Ok(iter)
	}

	fn progress_interval(&self) -> std::time::Duration {
		self.progress_interval
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
}
