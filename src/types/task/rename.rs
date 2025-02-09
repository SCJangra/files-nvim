use std::{path::PathBuf, sync::mpsc::Sender};

use nvim_oxi::libuv::AsyncHandle;

use crate::error::TaskError;

pub struct Rename {
	handler: AsyncHandle,
	sender: Sender<RenameResult>,
	ty: RenameType,
}

pub enum RenameType {
	Single { index: usize, file: PathBuf, to: String },
}

pub enum RenameResponse {
	Single { index: usize, name: String },
}

pub type RenameResult = Result<RenameResponse, TaskError>;

impl Rename {
	pub(crate) fn single(
		index: usize,
		file: PathBuf,
		to: String,
		handler: AsyncHandle,
		sender: Sender<RenameResult>,
	) -> Self {
		Self { handler, sender, ty: RenameType::Single { index, file, to } }
	}

	pub(crate) fn exec(self) {
		let res = match self.ty {
			RenameType::Single { index, file: path, to } => Self::rename_single(index, path, to),
		};

		self.sender.send(res).ok();
		self.handler.send().ok();
	}

	fn rename_single(index: usize, file: PathBuf, to_name: String) -> RenameResult {
		let from = file.clone();
		let mut to = file;
		to.set_file_name(&to_name);

		std::fs::rename(from, to)?;

		Ok(RenameResponse::Single { index, name: to_name })
	}
}
