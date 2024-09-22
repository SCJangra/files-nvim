use std::sync::{mpsc, Arc, LazyLock};

use dashmap::DashMap;
use nvim_oxi::{
	self as oxi,
	api::{self, Buffer},
	libuv::AsyncHandle,
};

use crate::{error::*, types::*, LogErr, CHANNEL};

use super::Task;

/// A map from [`Buffer`] to [`Explorer`] for all active explorers.
static OPEN_EXPS: LazyLock<DashMap<Buffer, Explorer>> = LazyLock::new(DashMap::new);

/// An active file explorer. This may or may not be attached to a window, but is always attached to
/// a buffer.
#[derive(Clone)]
pub struct Explorer {
	buf: Buffer,
	dir: ArcPath,
	files: ArcFiles,
}

/// How to open a new file explorer.
pub enum OpenIn {
	CurrentWin,
}

impl Explorer {
	/// Open new explorer in the current window.
	pub fn open_current(_: ()) {
		Self::open(OpenIn::CurrentWin).log_err().ok();
	}

	pub fn setup_keymaps(&mut self) -> Result<()> {
		Ok(())
	}

	/// List the files of `dir` in the explorer.
	pub fn list(&mut self, dir: ArcPath) -> Result<()> {
		let (sender, mut receiver) = mpsc::channel::<ListResult>();

		let mut buf = self.buf.clone();
		let handler = AsyncHandle::new(move || {
			Self::do_list(&mut buf, &mut receiver).log_err().ok();
		})?;

		let list = Task::List(List::new(dir, handler, sender));

		CHANNEL.send(list)?;

		Ok(())
	}

	fn do_list(buf: &mut Buffer, receiver: &mut mpsc::Receiver<ListResult>) -> Result<()> {
		let response = receiver.recv()??;
		let lines = response
			.files
			.iter()
			.map(|f| f.path.file_name().unwrap_or_default())
			.map(|name| oxi::String::from_bytes(name.as_encoded_bytes()));

		buf.set_lines(0.., true, lines)?;

		let mut exp = OPEN_EXPS.get_mut(buf).ok_or(Error::NoExplorer(buf.clone()))?;
		exp.dir = response.dir;
		exp.files = response.files;

		Ok(())
	}

	fn open(open: OpenIn) -> Result<()> {
		let buf = api::create_buf(true, true)?;
		let dir = std::env::current_dir()?;
		let dir = Arc::new(dir);

		let exp = Explorer { buf: buf.clone(), dir: Arc::clone(&dir), files: Arc::new(Vec::new()) };

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		win.set_buf(&buf)?;

		OPEN_EXPS.insert(buf.clone(), exp.clone());

		let mut exp = OPEN_EXPS.get_mut(&buf).ok_or(Error::NoExplorer(buf))?;

		exp.setup_keymaps()?;
		exp.list(dir)?;

		Ok(())
	}
}
