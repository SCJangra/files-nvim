use std::sync::LazyLock;

use dashmap::DashMap;
use nvim_oxi::{self as oxi, Dictionary, Function, Object};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use traits::*;
use types::*;

mod error;
mod impls;
mod traits;
mod types;

static CHANNEL: LazyLock<UnboundedSender<types::Task>> = LazyLock::new(|| {
	let (s, r) = unbounded_channel();
	std::thread::spawn(|| start(r));
	s
});

static DIR_CACHE: LazyLock<DashMap<ArcPath, ArcFiles>> = LazyLock::new(DashMap::new);

#[nvim_oxi::plugin]
fn files_nvim() -> oxi::Result<Dictionary> {
	Ok(Dictionary::from_iter([
		("set_config", Object::from(Function::from_fn(Config::set_config))),
		("get_config", Object::from(Function::from_fn(Config::get_config))),
		(
			"exp",
			Object::from(Dictionary::from_iter([(
				"open_current",
				Object::from(Function::from_fn(Explorer::open_current)),
			)])),
		),
	]))
}

#[tokio::main]
async fn start(mut channel: UnboundedReceiver<types::Task>) {
	while let Some(task) = channel.recv().await {
		match task {
			types::Task::List(list) => list.execute().await,
		}
	}
}
