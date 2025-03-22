use std::{path::PathBuf, sync::Arc};

use crate::{error::TaskError, types::File};

pub(crate) enum Msg {
	List(Vec<File>),
	TaskDone(usize),
	TaskError(usize, TaskError),
	Rename(usize, String),
	InsertFile(File, PathBuf),
	FileUpdated(Arc<PathBuf>),
}
