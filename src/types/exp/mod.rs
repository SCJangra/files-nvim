mod config;
mod nav;

pub use config::*;

use nav::*;

use std::{
	path::PathBuf,
	sync::{mpsc, LazyLock},
};

use dashmap::{
	mapref::one::{Ref, RefMut},
	DashMap,
};
use nvim_oxi::{
	self as nvim,
	api::{
		self,
		opts::{BufDeleteOpts, SetKeymapOpts},
		types::Mode,
		Buffer,
	},
	libuv::AsyncHandle,
};

use crate::{error::*, types::*, LIST};

/// A map from [`Buffer`] to [`Explorer`] for all active explorers.
static OPEN_EXPS: LazyLock<DashMap<Buffer, Explorer>> = LazyLock::new(DashMap::new);

/// An active file explorer. This may or may not be attached to a window, but is always attached to
/// a buffer.
pub struct Explorer {
	buf: Buffer,
	ns: u32,
	files: Vec<File>,
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

		let mode = Mode::Normal;

		self.buf.set_keymap(mode, &maps.quit, "", &Self::map_quit(self.buf))?;
		self.buf.set_keymap(mode, &maps.enter, "", &Self::map_enter(self.buf))?;
		self.buf.set_keymap(mode, &maps.next, "", &Self::map_next(self.buf))?;
		self.buf.set_keymap(mode, &maps.prev, "", &Self::map_prev(self.buf))?;

		Ok(())
	}

	/// List the files of `dir` in the explorer.
	fn list(&mut self, dir: PathBuf, nav: Nav) -> Result<()> {
		match nav {
			Nav::Next => self.nav.go_to_next(),
			Nav::Prev => self.nav.go_to_prev(),
			Nav::New => self.nav.insert(dir.clone()),
			Nav::Noop => (),
		}

		let (sender, receiver) = mpsc::channel::<ListResult>();

		let buf = self.buf;
		let handler = AsyncHandle::new(move || {
			let response = match receiver.recv()? {
				// Returning ok here because we don't want to log this error.
				Err(Error::Cancelled) => return Ok(()),
				res => res?,
			};

			let nav = nav.clone();

			nvim::schedule(move |_| Self::do_list(response, buf, nav).unwrap_or_default());

			Result::Ok(())
		})?;

		LIST.send(List::new(dir, handler, sender)).map_err(Error::SendList)?;

		Ok(())
	}

	fn do_list(response: ListResponse, buf: Buffer, _nav: Nav) -> Result<()> {
		let config = Config::arc_clone();

		let icons = response.files.iter().map(|f| config.icon(f));

		let lines = response.files.iter().zip(icons.clone()).map(|(file, icon)| {
			let name = file.path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
			nvim::string!("{:2} {}", icon.icon, name)
		});

		buf.set_lines(0.., true, lines)?;

		let ns = { Self::get(&buf)?.ns };
		buf.clear_namespace(ns, 0..)?;
		for (index, icon) in icons.enumerate() {
			buf.add_highlight(ns, &icon.name, index, 0..1).ok();
		}

		let mut exp = Self::get_mut(&buf)?;
		exp.files = response.files;

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
			FileType::DirectoryEmpty | FileType::DirectoryFull => self.list(file.path.clone(), Nav::New)?,
			// TODO: Open files
			// TODO: Follow symbolic links
			_ => nvim::print!("Unsupported operation"),
		};

		Ok(())
	}

	/// Go to the next directory.
	fn next(&mut self) -> Result<()> {
		let Some(dir) = self.nav.next() else { return Ok(()) };
		self.list(dir.clone(), Nav::Next)?;
		Ok(())
	}

	/// Go to the previous file.
	fn prev(&mut self) -> Result<()> {
		let Some(dir) = self.nav.prev() else { return Ok(()) };
		self.list(dir.clone(), Nav::Prev)?;
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
		let buf = api::create_buf(true, true)?;
		let ns = api::create_namespace(Self::NS);
		let dir = std::env::current_dir()?;

		let exp = Explorer { buf, ns, files: Vec::new(), nav: Navigator::new(dir.clone()) };

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		buf.set_option("filetype", Self::NAME)?;
		win.set_buf(&buf)?;

		Self::insert(buf, exp);

		let mut exp = Self::get_mut(&buf)?;

		exp.setup_keymaps()?;
		exp.list(dir, Nav::Noop)?;

		Ok(())
	}

	#[inline(always)]
	fn get_mut(buf: &Buffer) -> Result<RefMut<'_, Buffer, Self>> {
		OPEN_EXPS.get_mut(buf).ok_or_else(|| Error::NoExplorer(*buf))
	}

	#[inline(always)]
	fn get(buf: &Buffer) -> Result<Ref<'_, Buffer, Self>> {
		OPEN_EXPS.try_get(buf).try_unwrap().ok_or_else(|| Error::NoExplorer(*buf))
	}

	#[inline(always)]
	fn remove(buf: &Buffer) -> Result<Self> {
		OPEN_EXPS.remove(buf).ok_or_else(|| Error::NoExplorer(*buf)).map(|(_, exp)| exp)
	}

	#[inline(always)]
	fn insert(buf: Buffer, exp: Self) {
		OPEN_EXPS.insert(buf, exp);
	}
}
