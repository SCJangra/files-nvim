use std::sync::{Arc, LazyLock};

use nvim_oxi::{
	conversion::{FromObject, ToObject},
	Function, Object,
};
use serde::{Deserialize, Serialize};

use crate::{error::*, traits::*, types::*};

/// Global configuration of the plugin.
static mut CONFIG: LazyLock<Arc<Config>> = LazyLock::new(|| {
	Arc::new(Config {
		explorer: ExplorerConfig {
			keymaps: ExplorerKeymaps {
				quit: String::from("q"),
				enter: String::from("<CR>"),
				next: String::from("l"),
				prev: String::from("h"),
				up: String::from("<A-h>"),
				rename: String::from("r"),
				copy: String::from("y"),
			},
			fields: vec![Field::Size],
			name_width: 40,
			column_seperator: String::from(" "),
		},
		icons: Icons {
			file_name: Default::default(),
			extension: Default::default(),
			default: Icon { name: String::from("DevIconDefault"), icon: '' },
			dir_full: Icon { name: String::from(Explorer::DIR_HIGHLIGHT), icon: '' },
			dir_empty: Icon { name: String::from(Explorer::DIR_HIGHLIGHT), icon: '' },
		},
		input: None,
		get_mode: Function::from_fn(|_| {
			unimplemented!("pass the `get_mode` function in config");
			#[allow(unreachable_code)]
			0
		}),
	})
});

/// Configuration of this plugin.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
	pub explorer: ExplorerConfig,
	pub icons: Icons,
	pub input: Option<Input>,
	pub get_mode: Function<(), i32>,
}

impl Config {
	/// Get an [`Arc`] to the global config.
	pub fn arc_clone() -> Arc<Self> {
		unsafe { Arc::clone(&*CONFIG) }
	}

	/// Clone the global config.
	pub fn clone() -> Self {
		unsafe { (**CONFIG).clone() }
	}

	/// Get a global Neovim [`Object`] that represents the current configuration. This clones the
	/// global config.
	pub fn get_config(_: ()) -> Option<Object> {
		Self::clone().to_object().log_error().ok()
	}

	/// Replace the current configuration by the given [`Object`].
	pub fn set_config(object: Object) {
		let Ok(config) = Self::from_object(object).log_error() else { return };
		let config = Arc::new(config);
		unsafe {
			let ptr = (&*CONFIG) as *const _ as *mut Arc<Self>;
			std::ptr::replace(ptr, config)
		};
	}

	/// Returns the icon for a given file path.
	pub fn icon(&self, file: &File) -> &Icon {
		match file.ty {
			// TODO: Return a directory icon.
			FileType::DirectoryEmpty => return &self.icons.dir_empty,
			FileType::DirectoryFull => return &self.icons.dir_full,
			_ => { /* Continue below to minimize nesting */ },
		};

		let maybe_icon = file
			.path
			.extension()
			.and_then(|e| e.to_str())
			.and_then(|e| self.icons.extension.get(e));

		if let Some(icon) = maybe_icon {
			return icon;
		}

		let maybe_icon = file
			.path
			.file_name()
			.and_then(|n| n.to_str())
			.and_then(|n| self.icons.file_name.get(n));

		if let Some(icon) = maybe_icon {
			return icon;
		}

		&self.icons.default
	}

	pub fn input(&self) -> Result<&Input> {
		self.input.as_ref().ok_or_else(|| Error::NoInputFn)
	}

	pub fn get_mode(&self) -> Result<i32> {
		self.get_mode.call(()).map_err(Into::into)
	}
}

crate::lua_interop!(Config);
