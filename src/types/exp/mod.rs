mod config;
mod nav;
mod rename;

pub use config::*;

use nav::*;
use rayon::slice::ParallelSliceMut;

use std::{
	fmt::Write,
	ops::Range,
	path::{Path, PathBuf},
	sync::LazyLock,
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

use crate::{clipboard::*, error::*, msg::*, task, task_manager::*, traits::*, types::*, utils::fun};

/// A map from [`Buffer`] to [`Explorer`] for all active explorers.
static OPEN_EXPS: LazyLock<DashMap<Buffer, Explorer>> = LazyLock::new(DashMap::new);

/// An active file explorer. This may or may not be attached to a window, but is always attached to
/// a buffer.
pub struct Explorer {
	buf: Buffer,
	ns: u32,
	files: Vec<File>,
	nav: Navigator,
	task: TaskManager,
	cb: Clipboard,
}

/// How to open a new file explorer.
pub enum OpenIn {
	CurrentWin,
}

impl Explorer {
	/// Name of the explorer buffer.
	#[allow(unused)]
	pub const NAME: &str = "FilesNvim";

	/// Namespace used for highlights and extmarks in the explorer.
	pub const NS: &str = "FilesNvimExplorer";

	/// Highlight group name for a directory icon.
	pub const DIR_HIGHLIGHT: &str = "FilesNvimDirectoryIcon";

	pub(crate) fn with_selected<F>(&self, f: F) -> Result<()>
	where
		F: FnOnce(Range<usize>) -> Result<()> + 'static,
	{
		let mode = Config::arc_clone().get_mode()?;

		let normal = 110;
		let visual = 118;
		let visual_line = 86;
		let visual_block = 22;

		let buf = self.buf;

		if mode == normal {
			let current = self.current_index()?;
			let range = current..current + 1;

			nvim::schedule(move |_| f(range).unwrap_or_default());
		} else if mode == visual || mode == visual_line || mode == visual_block {
			api::input("<ESC>")?;

			nvim::schedule(move |_| {
				let Ok((from, _)) = buf.get_mark('<').map_err(Error::from) else { return };
				let from = from - 1;

				let Ok((to, _)) = buf.get_mark('>').map_err(Error::from) else { return };
				let to = to - 1;

				f(from..to + 1).unwrap_or_default();
			});
		} else {
			return Err(Error::InvalidMode);
		};

		Ok(())
	}

	/// Setup key mappings for this explorer.
	pub fn setup_keymaps(&self) -> Result<()> {
		// NOTE: Key maps won't automatically refresh if the configuration is changed.
		let maps = &Config::arc_clone().explorer.keymaps;

		let mode = Mode::Normal;

		self.buf.set_keymap(mode, &maps.quit, "", &Self::map_quit(self.buf))?;
		self.buf.set_keymap(mode, &maps.enter, "", &Self::map_enter(self.buf))?;
		self.buf.set_keymap(mode, &maps.next, "", &Self::map_next(self.buf))?;
		self.buf.set_keymap(mode, &maps.prev, "", &Self::map_prev(self.buf))?;
		self.buf.set_keymap(mode, &maps.up, "", &Self::map_up(self.buf))?;
		self.buf.set_keymap(mode, &maps.rename, "", &Self::map_rename(self.buf))?;
		self.buf.set_keymap(mode, &maps.create, "", &Self::map_create(self.buf))?;
		self.buf.set_keymap(mode, &maps.paste, "", &Self::map_paste(self.buf))?;

		self.buf.set_keymap(Mode::Normal, &maps.copy, "", &Self::map_copy(self.buf))?;
		self.buf.set_keymap(Mode::Visual, &maps.copy, "", &Self::map_copy(self.buf))?;

		self.buf.set_keymap(Mode::Normal, &maps.cut, "", &Self::map_cut(self.buf))?;
		self.buf.set_keymap(Mode::Visual, &maps.cut, "", &Self::map_cut(self.buf))?;

		self.buf
			.set_keymap(Mode::Normal, &maps.delete, "", &Self::map_delete(self.buf))?;
		self.buf
			.set_keymap(Mode::Visual, &maps.delete, "", &Self::map_delete(self.buf))?;

		Ok(())
	}

	/// Setup buffer options for this explorer.
	pub fn setup_opts(&self) -> Result<()> {
		self.buf.set_option("ma", false)?;
		Ok(())
	}

	/// List the files of `dir` in the explorer.
	fn list(&mut self, dir: PathBuf, nav: Nav) -> Result<()> {
		match nav {
			Nav::Next => self.nav.go_to_next(),
			Nav::Prev => self.nav.go_to_prev(),
			Nav::Up => self.nav.got_to_up(),
			Nav::New => self.nav.insert(dir.clone()),
			Nav::Noop => (),
		}

		self.task
			.spawn_atomic(task::List::new(dir), |(dir, files)| Msg::List { dir, files });

		Ok(())
	}

	/// Create a file or directory.
	fn create(&self) -> Result<()> {
		let dest = self.nav.current_dir().to_path_buf();
		let buf = self.buf;

		let on_path_input = nvim::Function::from_fn_once(move |maybe_path: Option<String>| {
			let Some(path) = maybe_path else { return };

			nvim::schedule(move |_| {
				let task = task::Create::new(path, dest.clone());

				// This call will deadlock without the above `nvim::schedule` wrap-up.
				let Ok(mut exp) = Self::get_mut(&buf).map_err(|_| Error::NoExplorer(buf)) else { return };
				exp.task.spawn_atomic(task, |_| Msg::DirUpdated { dir: dest });
			});
		});

		let opts = InputOpts { prompt: String::from("Name: "), default: String::new() };

		Config::arc_clone().input(opts, on_path_input)
	}

	/// Re-render the files in the explorer, this is called after renaming, creating, and deleting
	/// some files.
	fn refresh(&mut self) -> Result<()> {
		let config = Config::arc_clone();

		self.files
			.par_sort_by(|a, b| (!a.is_dir(), a.name()).cmp(&(!b.is_dir(), b.name())));

		let lines = self.files.iter().map(|file| {
			let mut b = nvim::StringBuilder::new();

			let name = file.path.file_name().unwrap_or_default().to_str().unwrap_or_default();
			let icon = config.icon(file);
			let width = config.explorer.name_width;

			let (name, dots) = match name.len() > width {
				true => (&name[..width.saturating_sub(2)], ".."),
				false => (name, ""),
			};

			b.write_fmt(format_args!("{:2} {name:1$}{dots}", icon.icon, width - dots.len()))
				.map_err(Error::from)
				.ok();

			config.explorer.fields.iter().for_each(|field| {
				b.write_str(&config.explorer.column_seperator).map_err(Error::from).ok();
				match field {
					Field::Size => {
						let (value, unit) = fun::bytes_to_size(file.size);
						let val = (value * 100.0) / 100.0;

						b.write_fmt(format_args!("{val:>6.2} {unit}")).map_err(Error::from).ok();
					},
				}
			});

			b.finish()
		});

		self.buf
			.with_modifiable(|| self.buf.set_lines(0.., true, lines).map_err(Into::into))?;

		self.buf.clear_namespace(self.ns, 0..)?;
		for (index, icon) in self.files.iter().map(|f| config.icon(f)).enumerate() {
			self.buf.add_highlight(self.ns, &icon.name, index, 0..1).ok();
		}

		Ok(())
	}

	/// Exit this explorer.
	fn quit(self) -> Result<()> {
		self.buf
			.delete(&BufDeleteOpts::builder().force(true).build())
			.map_err(Into::into)
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

		let mut win = match open {
			OpenIn::CurrentWin => api::get_current_win(),
		};

		win.set_buf(&buf)?;

		let (msg_sender, msg_receiver) = crossbeam_channel::unbounded::<Msg>();
		let handle = AsyncHandle::new(move || {
			while let Ok(msg) = msg_receiver.try_recv() {
				nvim::schedule(move |_| Self::update(buf, msg).unwrap_or_default());
			}

			Result::Ok(())
		})?;

		let mut exp = Explorer {
			buf,
			ns,
			files: Vec::new(),
			nav: Navigator::new(dir.clone()),
			task: TaskManager::new(handle, msg_sender),
			cb: Clipboard::new(),
		};

		exp.setup_keymaps()?;
		exp.setup_opts()?;
		exp.list(dir, Nav::Noop)?;

		Self::insert(buf, exp);

		Ok(())
	}

	fn update(buf: Buffer, msg: Msg) -> Result<()> {
		let mut exp = Self::get_mut(&buf)?;

		match msg {
			Msg::List { dir, files } => {
				if dir == exp.current_dir() {
					exp.files = files;
					exp.refresh()?;
				}

				Ok(())
			},
			Msg::TaskDone { index } => {
				exp.task.remove_task(index);
				Ok(())
			},
			Msg::TaskError { error, .. } => Err(error.into()),
			Msg::DirUpdated { dir } => {
				if dir == exp.nav.current_dir() {
					exp.list(dir, Nav::Noop)?;
				}

				Ok(())
			},
			Msg::Noop => Ok((/* Do nothing */)),
		}
	}

	fn copy(&mut self, mut indices: Range<usize>) -> Result<()> {
		indices.try_for_each(|index| {
			let file = self.files.get(index).ok_or_else(|| Error::NoFile(index))?;

			self.cb.cut.remove(file);

			match self.cb.copy.contains(file) {
				true => self.cb.copy.remove(file),
				false => self.cb.copy.insert(file.clone()),
			};

			Result::Ok(())
		})
	}

	fn cut(&mut self, mut indices: Range<usize>) -> Result<()> {
		indices.try_for_each(|index| {
			let file = self.files.get(index).ok_or_else(|| Error::NoFile(index))?;

			self.cb.copy.remove(file);

			match self.cb.cut.contains(file) {
				true => self.cb.cut.remove(file),
				false => self.cb.cut.insert(file.clone()),
			};

			Result::Ok(())
		})
	}

	fn delete(&mut self, mut indices: Range<usize>) -> Result<()> {
		let count = indices.len();

		let prompt = format!("Delete {count} files?");
		let options = String::from("&Yes\n&No");

		let choice = Config::arc_clone().confirm(prompt, options, 2, String::from("Question"))?;

		if choice != 1 {
			return Ok(());
		}

		let files = indices.try_fold(Vec::with_capacity(count), |mut acc, index| {
			let file = self.files.get(index).ok_or_else(|| Error::NoFile(index))?;
			acc.push(file.clone());
			Result::Ok(acc)
		})?;

		let dir = self.current_dir().to_path_buf();

		let task = task::Delete::new(files);
		self.task.spawn(task, move |_| Msg::DirUpdated { dir: dir.clone() });

		Ok(())
	}

	pub fn paste(&mut self) -> Result<()> {
		let files = self.cb.copy.iter().cloned().collect::<Vec<_>>();
		let dest = self.nav.current_dir().to_path_buf();
		let buf = self.buf;

		self.cb.copy = Default::default();

		let task = task::Copy::new(files, dest.clone());

		self.task.spawn(task, move |_| {
			let Ok(exp) = Self::get(&buf) else { return Msg::Noop };
			let current_dir = exp.nav.current_dir();

			match current_dir.starts_with(&dest) {
				true => Msg::DirUpdated { dir: current_dir.to_path_buf() },
				false => Msg::Noop,
			}
		});

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

// Maps
impl Explorer {
	fn map_enter(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.enter()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_next(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.next()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_prev(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.prev()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_up(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.up()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_quit(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::remove(&buf).and_then(|exp| exp.quit()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_rename(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get(&buf).and_then(|exp| exp.rename()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_create(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get(&buf).and_then(|exp| exp.create()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_copy(buf: Buffer) -> SetKeymapOpts {
		let cb = move |indices| {
			let mut exp = Self::get_mut(&buf)?;
			exp.copy(indices)
		};

		let cb = move |_| Self::get(&buf).and_then(|exp| exp.with_selected(cb)).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_cut(buf: Buffer) -> SetKeymapOpts {
		let cb = move |indices| {
			let mut exp = Self::get_mut(&buf)?;
			exp.cut(indices)
		};

		let cb = move |_| Self::get(&buf).and_then(|exp| exp.with_selected(cb)).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_delete(buf: Buffer) -> SetKeymapOpts {
		let cb = move |indices| {
			let mut exp = Self::get_mut(&buf)?;
			exp.delete(indices)
		};

		let cb = move |_| Self::get(&buf).and_then(|exp| exp.with_selected(cb)).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}

	fn map_paste(buf: Buffer) -> SetKeymapOpts {
		let cb = move |_| Self::get_mut(&buf).and_then(|mut exp| exp.paste()).unwrap_or_default();
		SetKeymapOpts::builder().callback(cb).build()
	}
}

// Navigation
impl Explorer {
	/// Go to the next directory.
	fn next(&mut self) -> Result<()> {
		let Some(dir) = self.nav.next() else { return Ok(()) };
		self.list(dir.to_path_buf(), Nav::Next)
	}

	/// Go to the previous file.
	fn prev(&mut self) -> Result<()> {
		let Some(dir) = self.nav.prev() else { return Ok(()) };
		self.list(dir.to_path_buf(), Nav::Prev)
	}

	fn up(&mut self) -> Result<()> {
		let Some(dir) = self.nav.up() else { return Ok(()) };
		self.list(dir.to_path_buf(), Nav::Up)
	}

	fn current_file(&self) -> Result<&File> {
		let index = self.current_index()?;
		let file = self.files.get(index).ok_or_else(|| Error::NoFile(index))?;

		Ok(file)
	}

	fn current_index(&self) -> Result<usize> {
		let win = api::get_current_win();
		let buf = win.get_buf()?;

		if buf != self.buf {
			return Err(Error::BufferMismatch);
		}

		// 0 is row, and row is 1-indexed
		let index = win.get_cursor()?.0.saturating_sub(1);

		Ok(index)
	}

	fn current_dir(&self) -> &Path {
		self.nav.current_dir()
	}

	/// Open the file or enter the directory under cursor.
	fn enter(&mut self) -> Result<()> {
		let file = self.current_file()?;

		match file.ty {
			FileType::DirectoryEmpty | FileType::DirectoryFull => self.list(file.path.clone(), Nav::New)?,
			FileType::File => file.open()?,
			// TODO: Follow symbolic links
			_ => nvim::print!("Unsupported operation"),
		};

		Ok(())
	}
}
