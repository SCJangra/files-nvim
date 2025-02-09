use std::sync::LazyLock;

use crossbeam_channel::{unbounded, Receiver, Sender};
use nvim_oxi::{Dictionary, Function, Object};

mod error;
mod impls;
mod traits;
mod types;
mod utils;

pub(crate) struct Channels {
	pub(crate) list: Sender<types::List>,
	pub(crate) rename: Sender<types::Rename>,
}

static CHANNELS: LazyLock<Channels> = LazyLock::new(|| {
	let (list_s, list_r) = unbounded();
	std::thread::spawn(|| list(list_r));

	let (rename_s, rename_r) = unbounded();
	std::thread::spawn(|| rename(rename_r));

	Channels { list: list_s, rename: rename_s }
});

#[nvim_oxi::plugin]
fn files_nvim() -> types::Result<Dictionary> {
	Ok(Dictionary::from_iter([
		("set_config", Object::from(Function::from_fn(types::Config::set_config))),
		("get_config", Object::from(Function::from_fn(types::Config::get_config))),
		(
			"exp",
			Object::from(Dictionary::from_iter([(
				"open_current",
				Object::from(Function::from_fn(types::Explorer::open_current)),
			)])),
		),
	]))
}

fn list(r: Receiver<types::List>) {
	while let Ok(l) = r.recv() {
		l.exec(&r);
	}
}

fn rename(r: Receiver<types::Rename>) {
	while let Ok(rename) = r.recv() {
		rename.exec();
	}
}
