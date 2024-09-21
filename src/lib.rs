use nvim_oxi::{self as oxi, Dictionary, Function, Object};

use types::*;

mod types;

#[nvim_oxi::plugin]
fn files_nvim() -> oxi::Result<Dictionary> {
	Ok(Dictionary::from_iter([
		("set_config", Object::from(Function::from_fn(Config::set_config))),
		("get_config", Object::from(Function::from_fn(Config::get_config))),
	]))
}
