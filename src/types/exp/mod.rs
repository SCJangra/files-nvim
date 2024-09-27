mod config;
mod nav;

pub use config::*;

use nav::*;

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

use crate::{error::*, types::*, CHANNEL, DIR_CACHE};

/// A map from [`Buffer`] to [`Explorer`] for all active explorers.
static OPEN_EXPS: LazyLock<DashMap<Buffer, Explorer>> = LazyLock::new(DashMap::new);

/// An active file explorer. This may or may not be attached to a window, but is always attached to
/// a buffer.
pub struct Explorer {
	buf: Buffer,
	ns: u32,
	nav: Navigator,
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
		self.buf.set_keymap(mode, &maps.next, "", &Self::map_next(buf.clone()))?;
		self.buf.set_keymap(mode, &maps.prev, "", &Self::map_prev(buf.clone()))?;

		Ok(())
	}

	/// List the files of `dir` in the explorer.
	fn list(&mut self, dir: ArcPath, nav: Nav) -> Result<()> {
		let (sender, receiver) = mpsc::channel::<ListResult>();

		let buf = self.buf.clone();
		let handler = AsyncHandle::new(move || {
			let response = receiver.recv()??;

			let buf = buf.clone();
			let nav = nav.clone();

			nvim_oxi::schedule(move |_| Self::do_list(response, buf, nav).unwrap_or_default());

			Result::Ok(())
		})?;
		let list = Task::List(List::new(dir, handler, sender));

		CHANNEL.send(list)?;

		Ok(())
	}

	fn do_list(response: ListResponse, mut buf: Buffer, nav: Nav) -> Result<()> {
		let config = Config::arc_clone();

		let icons = response.files.iter().map(|f| config.icon(f));

		let lines = response.files.iter().zip(icons.clone()).map(|(file, icon)| {
			let name = file.path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
			// TODO: Directly build a Neovim String
			let str = format!("{:2} {}", icon.icon, name);
			nvim_oxi::String::from_bytes(str.as_bytes())
		});

		buf.set_lines(0.., true, lines)?;

		let ns = {
			let mut exp = Self::get_mut(&buf)?;

			match nav {
				Nav::Next => exp.nav.go_to_next(),
				Nav::Prev => exp.nav.go_to_prev(),
				Nav::New => exp.nav.insert(response.dir),
				Nav::Noop => (),
			}

			exp.ns
		};

		for (index, icon) in icons.enumerate() {
			buf.add_highlight(ns, &icon.name, index, 0..1).ok();
		}

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
		let files = self.files()?;
		let file = files.get(index).ok_or_else(|| Error::NoFile(index))?;

		match file.ty {
			FileType::DirectoryEmpty | FileType::DirectoryFull => self.list(Arc::clone(&file.path), Nav::New)?,
			// TODO: Open files
			// TODO: Follow symbolic links
			_ => oxi::print!("Unsupported operation"),
		};

		Ok(())
	}

	/// Go to the next directory.
	fn next(&mut self) -> Result<()> {
		let Some(dir) = self.nav.next() else { return Ok(()) };
		self.list(Arc::clone(dir), Nav::Next)?;
		Ok(())
	}

	/// Go to the previous file.
	fn prev(&mut self) -> Result<()> {
		let Some(dir) = self.nav.prev() else { return Ok(()) };
		self.list(Arc::clone(dir), Nav::Prev)?;
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

	fn map_next(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.next());
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_prev(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.prev());
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

		let exp = Explorer { buf: buf.clone(), ns, nav: Navigator::new(Arc::clone(&dir)) };

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		buf.set_option("filetype", Self::NAME)?;
		win.set_buf(&buf)?;

		Self::insert(buf.clone(), exp);

		let mut exp = Self::get_mut(&buf)?;

		exp.setup_keymaps()?;
		exp.list(Arc::clone(&dir), Nav::Noop)?;

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

	/// Get current files of the explorer.
	fn files(&self) -> Result<ArcFiles> {
		let dir = self.nav.current();

		DIR_CACHE
			.try_get(self.nav.current())
			.try_unwrap()
			.map(|files| Arc::clone(&files))
			.ok_or_else(|| Error::NoFiles(Arc::clone(dir)))
	}
}
