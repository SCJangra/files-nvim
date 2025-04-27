mod config;

pub use config::*;

use std::{
	any::{Any, TypeId},
	collections::BTreeMap,
	sync::Arc,
};

use crossbeam_channel::Sender;
use nvim_oxi::{
	api::{
		self,
		opts::{BufDeleteOpts, SetExtmarkOpts},
		Buffer, Window,
	},
	libuv::AsyncHandle,
};

use crate::{
	msg::Msg,
	traits::{AtomicTask, IterExt, Task, TaskHandle},
	types::{Config, OpenIn, Result},
};

type ArcTaskHandle = Arc<dyn TaskHandle + Send + Sync>;

pub(crate) struct TaskManager {
	index: usize,
	tasks: BTreeMap<usize, ArcTaskHandle>,
	unique_tasks: BTreeMap<TypeId, ArcTaskHandle>,
	handle: AsyncHandle,
	msg: Sender<Msg>,
	view: Option<View>,
	ns: u32,
}

struct View {
	#[allow(unused)]
	win: Window,
	buf: Buffer,
}

impl TaskManager {
	/// Namespace used for highlights and extmarks in the task manager.
	pub const NS: &str = "FilesNvimTaskManager";

	/// Highlight group for task headers.
	pub const HEAD_HL: &str = "FilesNvimTaskHead";

	/// Highlight group for task body.
	pub const BODY_HL: &str = "FilesNvimTaskBody";

	pub(crate) fn new(handle: AsyncHandle, msg: Sender<Msg>) -> Result<Self> {
		Ok(Self {
			index: 0,
			tasks: BTreeMap::new(),
			unique_tasks: BTreeMap::new(),
			view: None,
			handle,
			msg,
			ns: api::create_namespace(Self::NS),
		})
	}

	pub(crate) fn spawn<T, M>(&mut self, task: T, msg: M)
	where
		T: Task + TaskHandle + Any + Send + Sync + 'static,
		M: Fn(T::Update) -> Msg + Send + Sync + 'static,
	{
		let task = Arc::new(task);
		let index = self.add_task(task.clone() as ArcTaskHandle);
		let message = self.msg.clone();
		let handle = self.handle.clone();

		rayon::spawn(move || {
			let iter = match task.execute() {
				Ok(iter) => iter,
				Err(error) => {
					message.send(Msg::TaskError { index, error }).ok();
					handle.send().ok();
					return;
				},
			};

			let progress_interval = { Config::arc_clone().task_manager.progress.interval };

			iter.for_each_interval(progress_interval, |u| {
				let m = match u {
					Ok(u) => msg(u),
					Err(error) => Msg::TaskError { index, error },
				};

				message.send(m).ok();
				handle.send().ok();
			});

			message.send(Msg::TaskDone { index }).ok();
			handle.send().ok();
		});
	}

	pub(crate) fn spawn_atomic<T, M>(&mut self, task: T, msg: M)
	where
		T: AtomicTask + TaskHandle + Any + Send + Sync + 'static,
		M: FnOnce(T::Response) -> Msg + Send + Sync + 'static,
	{
		let task = Arc::new(task);
		let index = self.add_task(task.clone() as ArcTaskHandle);
		let msg_sender = self.msg.clone();
		let handle = self.handle.clone();

		rayon::spawn(move || {
			let res = match task.execute() {
				Ok(res) => res,
				Err(error) => {
					msg_sender.send(Msg::TaskError { index, error }).ok();
					msg_sender.send(Msg::TaskDone { index }).ok();
					handle.send().ok();
					return;
				},
			};

			let res = msg(res);
			msg_sender.send(res).ok();
			msg_sender.send(Msg::TaskDone { index }).ok();
			handle.send().ok();
		});
	}

	fn add_task(&mut self, task: ArcTaskHandle) -> usize {
		let index = self.index;

		if task.is_unique() {
			let id = task.type_id();

			if let Some(old) = self.unique_tasks.insert(id, task.clone()) {
				old.cancel();
			}
		}

		self.tasks.insert(index, task);
		self.index += 1;

		index
	}

	pub(crate) fn remove_task(&mut self, index: usize) {
		let Some(task) = self.tasks.remove(&index) else { return };

		if task.is_unique() {
			let id = task.type_id();
			self.unique_tasks.remove(&id);
		}
	}

	/// Quit the task manager and delete the underlying buffer.
	pub(crate) fn quit(self) -> Result<()> {
		let Some(view) = self.view else { return Ok(()) };
		view.buf.delete(&BufDeleteOpts::builder().force(true).build())?;
		Ok(())
	}

	/// Show the task manager.
	pub(crate) fn show(&mut self, open: OpenIn) -> Result<()> {
		if let Some(ref view) = self.view {
			view.buf.delete(&BufDeleteOpts::builder().force(true).build())?
		}

		let buf = api::create_buf(true, true)?;

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		win.set_buf(&buf)?;

		self.view = Some(View { buf, win });

		Ok(())
	}

	/// Refresh the task manager. This is a no-op if the task manager is hidden or closed.
	pub(crate) fn refresh(&self) -> Result<()> {
		let Some(view) = &self.view else { return Ok(()) };

		let width = { Config::arc_clone().win_width(view.win.handle())? };

		view.buf.clear_namespace(self.ns, ..)?;

		let mut count = 0;

		for (line, (_, task)) in self.tasks.iter().enumerate() {
			let mut lines = task.progress(width);

			if lines.is_empty() {
				continue;
			}

			let head = lines.remove(0);
			let body = lines.into_iter().map(|line| (line, Self::BODY_HL)).map(|c| [c]);

			let opts = SetExtmarkOpts::builder()
				.end_row(line + 1)
				.end_col(0)
				.hl_group(Self::HEAD_HL)
				.hl_eol(true)
				.virt_lines(body)
				.build();

			view.buf.set_lines(line..=line, true, [head])?;
			view.buf.set_extmark(self.ns, line, 0, &opts)?;

			count += 1;
		}

		// Clear remaining lines
		view.buf.set_lines(count.., true, Vec::<String>::new())?;

		Ok(())
	}
}
