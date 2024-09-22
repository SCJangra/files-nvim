mod config;

pub use config::*;

use std::sync::{mpsc, Arc, LazyLock};

use dashmap::{mapref::one::RefMut, DashMap};
use nvim_oxi::{
	self as oxi,
	api::{
		self,
		opts::{BufDeleteOpts, SetKeymapOpts},
		types::Mode,
		Buffer,
	},
	libuv::AsyncHandle,
};

use crate::{error::*, types::*, CHANNEL};

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
	/// Setup key mappings for this explorer.
	pub fn setup_keymaps(&mut self) -> Result<()> {
		// NOTE: Key maps won't automatically refresh if the configuration is changed.
		let maps = &Config::arc_clone().explorer.keymaps;

		let buf = self.buf.clone();
		let mode = Mode::Normal;

		self.buf.set_keymap(mode, &maps.quit, "", &Self::map_quit(buf.clone()))?;
		self.buf.set_keymap(mode, &maps.enter, "", &Self::map_enter(buf.clone()))?;

		Ok(())
	}

	/// List the files of `dir` in the explorer.
	pub fn list(&mut self, dir: ArcPath) -> Result<()> {
		let (sender, receiver) = mpsc::channel::<ListResult>();

		let mut buf = self.buf.clone();
		let handler = AsyncHandle::new(move || {
			let response = receiver.recv()??;
			let lines = response
				.files
				.iter()
				.map(|f| f.path.file_name().unwrap_or_default())
				.map(|name| oxi::String::from_bytes(name.as_encoded_bytes()));

			buf.set_lines(0.., true, lines)?;

			let mut exp = Self::get_mut(&buf)?;
			exp.dir = response.dir;
			exp.files = response.files;

			Ok::<_, Error>(())
		})?;

		let list = Task::List(List::new(dir, handler, sender));

		CHANNEL.send(list)?;

		Ok(())
	}

	pub fn enter(&mut self) -> Result<()> {
		let win = api::get_current_win();
		let buf = win.get_buf()?;

		if buf != self.buf {
			return Ok(());
		}

		// 0 is row, and row is 1-indexed
		let index = win.get_cursor()?.0.saturating_sub(1);
		let file = self.files.get(index).ok_or_else(|| Error::NoFile(index))?;

		match file.ty {
			FileType::Directory => self.list(Arc::clone(&file.path))?,
			// TODO: Open files
			// TODO: Follow symbolic links
			_ => oxi::print!("Unsupported operation"),
		};

		Ok(())
	}

	pub fn quit(self) -> Result<()> {
		self.buf
			.delete(&BufDeleteOpts::builder().force(true).build())
			.map_err(Into::into)
	}

	fn map_enter(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| {
			let Ok(mut exp) = Self::get_mut(&buf) else { return };
			exp.enter().ok();
		};

		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_quit(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::remove(&buf).and_then(|exp| exp.quit());
		SetKeymapOpts::builder().callback(cb).build()
	}

	/// Launch a new instance of the ['explorer'](Explorer) in the current window.
	pub fn open_current(_: ()) {
		Self::open(OpenIn::CurrentWin).ok();
	}

	/// Launch a new instance of the [`explorer`](Explorer).
	fn open(open: OpenIn) -> Result<()> {
		let mut buf = api::create_buf(true, true)?;
		let dir = std::env::current_dir()?;
		let dir = Arc::new(dir);

		let exp = Explorer { buf: buf.clone(), dir: Arc::clone(&dir), files: Arc::new(Vec::new()) };

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		buf.set_name("FilesNvim")?;
		win.set_buf(&buf)?;

		OPEN_EXPS.insert(buf.clone(), exp.clone());

		let mut exp = OPEN_EXPS.get_mut(&buf).ok_or_else(|| Error::NoExplorer(buf))?;

		exp.setup_keymaps()?;
		exp.list(dir)?;

		Ok(())
	}

	#[inline(always)]
	fn get_mut(buf: &Buffer) -> Result<RefMut<'_, Buffer, Self>> {
		OPEN_EXPS.get_mut(buf).ok_or_else(|| Error::NoExplorer(buf.clone()))
	}

	#[inline(always)]
	fn remove(buf: &Buffer) -> Result<Self> {
		OPEN_EXPS
			.remove(buf)
			.ok_or_else(|| Error::NoExplorer(buf.clone()))
			.map(|(_, exp)| exp)
	}
}
