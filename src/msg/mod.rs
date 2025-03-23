use std::path::PathBuf;

use crate::{error::TaskError, types::File};

pub(crate) enum Msg {
	/// List files in the explorer.
	List {
		/// Which directory to list in. The files will be listed only if the explorer is currently
		/// showing this directory.
		dir: PathBuf,
		/// The files to list
		files: Vec<File>,
	},
	/// A task is completed
	TaskDone {
		index: usize,
	},
	/// A task has produced an error.
	TaskError {
		index: usize,
		error: TaskError,
	},
	/// A directory is updated
	DirUpdated {
		dir: PathBuf,
	},
	Noop,
}
