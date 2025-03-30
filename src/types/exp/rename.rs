use nvim_oxi::{self as nvim};

use crate::{error::Error, msg::Msg, task, types::*};

impl Explorer {
	pub(crate) fn rename(&self) -> Result<()> {
		let file = self.current_file()?;

		let name = file.name_str().unwrap_or_default().to_string();
		let file = file.path.clone();
		let buf = self.buf;

		let on_new_name = nvim::Function::from_fn_once(move |maybe_name: Option<String>| {
			let Some(new_name) = maybe_name else { return Result::Ok(()) };

			nvim::schedule(move |_| {
				let dir = file.parent().map(|p| p.to_path_buf());
				let task = task::Rename::new(file, new_name);

				// This call will deadlock without the above `nvim::schedule` wrap-up.
				let Ok(mut exp) = Self::get_mut(&buf).map_err(|_| Error::NoExplorer(buf)) else { return };
				exp.task.spawn_atomic(task, |_| match dir {
					Some(dir) => Msg::DirUpdated { dir },
					None => Msg::Noop,
				});
			});
			Ok(())
		});

		let opts = InputOpts { prompt: String::from("Rename: "), default: name };

		Config::arc_clone().input(opts, on_new_name)
	}
}
