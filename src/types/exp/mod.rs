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
	ns: u32,
	dir: ArcPath,
	files: ArcFiles,
}

/// How to open a new file explorer.
pub enum OpenIn {
	CurrentWin,
}

impl Explorer {
	/// Name of the explorer buffer.
	pub const NAME: &str = "FilesNvim";

	/// Namespace used for highlights and extmarks in the explorer.
	pub const NS: &str = "FilesNvimExplorer";

	/// Highlight group name for a directory icon.
	pub const DIR_HIGHLIGHT: &str = "FilesNvimDirectoryIcon";

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
	fn list(&mut self, dir: ArcPath) -> Result<()> {
		let (sender, receiver) = mpsc::channel::<ListResult>();

		let mut buf = self.buf.clone();
		let handler = AsyncHandle::new(move || {
			let response = receiver.recv()??;

			let config = Config::arc_clone();

			let icons = response.files.iter().map(|f| config.icon(f));

			let lines = response.files.iter().zip(icons.clone()).map(|(file, icon)| {
				let name = file.path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
				// TODO: Directly build a Neovim String
				format!("{:2} {}", icon.icon, name)
			});

			buf.set_lines(0.., true, lines)?;

			let mut exp = Self::get_mut(&buf)?;
			exp.dir = response.dir;
			exp.files = Arc::clone(&response.files);

			let mut buf = buf.clone();
			for (index, icon) in icons.enumerate() {
				buf.add_highlight(exp.ns, &icon.name, index, 0..1).ok();
			}

			Ok::<_, Error>(())
		})?;

		let list = Task::List(List::new(dir, handler, sender));

		CHANNEL.send(list)?;

		Ok(())
	}

	/// Open the file or enter the directory under cursor.
	fn enter(&mut self) -> Result<()> {
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

	/// Exit this explorer.
	fn quit(self) -> Result<()> {
		self.buf
			.delete(&BufDeleteOpts::builder().force(true).build())
			.map_err(Into::into)
	}

	fn map_enter(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.enter());
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_quit(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::remove(&buf).and_then(|exp| exp.quit());
		SetKeymapOpts::builder().callback(cb).build()
	}

	/// Launch a new instance of the explorer in the current window.
	pub fn open_current(_: ()) {
		Self::open(OpenIn::CurrentWin).ok();
	}

	/// Launch a new instance of the explorer.
	fn open(open: OpenIn) -> Result<()> {
		let mut buf = api::create_buf(true, true)?;
		let ns = api::create_namespace(Self::NS);
		let dir = std::env::current_dir()?;
		let dir = Arc::new(dir);

		let exp = Explorer { buf: buf.clone(), dir: Arc::clone(&dir), files: Arc::new(Vec::new()), ns };

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		buf.set_name(Self::NAME)?;
		win.set_buf(&buf)?;

		Self::insert(buf.clone(), exp);

		let mut exp = Self::get_mut(&buf)?;

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

	#[inline(always)]
	fn insert(buf: Buffer, exp: Self) {
		OPEN_EXPS.insert(buf, exp);
	}
}
