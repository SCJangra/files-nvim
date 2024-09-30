use std::sync::LazyLock;

use crossbeam_channel::{unbounded, Receiver, Sender};
use nvim_oxi::{Dictionary, Function, Object};

use traits::*;
use types::*;

mod error;
mod impls;
mod traits;
mod types;
mod utils;

static LIST: LazyLock<Sender<List>> = LazyLock::new(|| {
	let (s, r) = unbounded();
	std::thread::spawn(|| list(r));
	s
});

#[nvim_oxi::plugin]
fn files_nvim() -> Result<Dictionary> {
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

fn list(r: Receiver<List>) {
	while let Ok(l) = r.recv() {
		l.exec(&r);
	}
}
