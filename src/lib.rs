use nvim_oxi::{Dictionary, Function, Object};

mod clipboard;
mod error;
mod impls;
mod msg;
mod task;
mod task_manager;
mod traits;
mod types;
mod utils;

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
