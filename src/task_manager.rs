use std::{
	any::{Any, TypeId},
	collections::BTreeMap,
	sync::Arc,
};

use crossbeam_channel::Sender;
use nvim_oxi::libuv::AsyncHandle;
use rayon::iter::ParallelIterator;

use crate::{
	msg::{Msg, MsgResult},
	traits::{AtomicTask, Task, TaskHandle},
};

type ArcTaskHandle = Arc<dyn TaskHandle + Send + Sync>;

pub(crate) struct TaskManager {
	index: usize,
	tasks: BTreeMap<usize, ArcTaskHandle>,
	unique_tasks: BTreeMap<TypeId, ArcTaskHandle>,
	handle: AsyncHandle,
	msg: Sender<MsgResult>,
}

impl TaskManager {
	pub(crate) fn new(handle: AsyncHandle, msg: Sender<MsgResult>) -> Self {
		Self { index: 0, tasks: BTreeMap::new(), unique_tasks: BTreeMap::new(), handle, msg }
	}

	pub(crate) fn spawn<T, M>(&mut self, task: T, msg: M)
	where
		T: Task + TaskHandle + Any + Send + Sync + 'static,
		M: Fn(Option<T::Progress>) + Send + Sync + 'static,
	{
		let task = Arc::new(task);
		let index = self.add_task(task.clone() as ArcTaskHandle);
		let done = self.msg.clone();

		rayon::spawn(move || {
			task.execute().for_each(|p| msg(Some(p)));
			msg(None);
			done.send(Ok(Msg::TaskDone(index))).ok();
		});
	}

	pub(crate) fn spawn_atomic<T, M>(&mut self, task: T, msg: M)
	where
		T: AtomicTask + TaskHandle + Any + Send + Sync + 'static,
		M: FnOnce(T::Response) -> MsgResult + Send + Sync + 'static,
	{
		let task = Arc::new(task);
		let index = self.add_task(task.clone() as ArcTaskHandle);
		let msg_sender = self.msg.clone();
		let handle = self.handle.clone();

		rayon::spawn(move || {
			let res = task.execute();
			let res = msg(res);
			msg_sender.send(res).ok();
			msg_sender.send(Ok(Msg::TaskDone(index))).ok();
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
}
