use std::sync::mpsc;

use nvim_oxi::{self as nvim, api::Buffer, libuv::AsyncHandle};

use crate::{error::*, types::*, CHANNELS};

impl Explorer {
	pub(crate) fn rename(&self) -> Result<()> {
		let index = self.current_index()?;
		let file = self.current_file()?;

		let name = file.name_str().unwrap_or_default().to_string();
		let file = file.path.clone();

		let (sender, receiver) = mpsc::channel::<RenameResult>();

		let handler = AsyncHandle::new(Self::on_renamed(self.buf, receiver))?;

		let on_new_name = nvim::Function::from_fn_once(move |maybe_name: Option<String>| {
			let Some(new_name) = maybe_name else { return Result::Ok(()) };

			CHANNELS
				.rename
				.send(Rename::single(index, file, new_name, handler, sender))
				.map_err(Error::SendRename)
		});

		let opts = InputOpts { prompt: String::from("Rename: "), default: name };

		Config::arc_clone().input()?.call((opts, on_new_name))?;

		Ok(())
	}

	fn on_renamed(buf: Buffer, receiver: mpsc::Receiver<RenameResult>) -> impl FnMut() -> Result<()> {
		move || {
			let response = receiver.recv()??;

			let mut exp = Self::get_mut(&buf)?;

			match response {
				RenameResponse::Single { index, name } => {
					exp.files
						.get_mut(index)
						.ok_or_else(|| Error::NoFile(index))?
						.path
						.set_file_name(name);
				},
			}

			nvim::schedule(move |_| Self::get_mut(&buf).and_then(|mut exp| exp.refresh()).unwrap_or_default());

			Ok(())
		}
	}
}
