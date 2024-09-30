use nvim_oxi::{self as nvim, api::Buffer};

use crate::{types::Task, LogErr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Io({0})")]
	Io(#[from] tokio::io::Error),

	#[error("Nvim({0})")]
	Nvim(#[from] nvim::Error),

	#[error("SendTask({0:?})")]
	SendTask(#[from] tokio::sync::mpsc::error::SendError<Task>),

	#[error("RecvError({0})")]
	RecvResult(#[from] std::sync::mpsc::RecvError),

	#[error("NoExplorer({0})")]
	NoExplorer(Buffer),

	#[error("NoFile({0})")]
	NoFile(usize),
}

impl From<nvim::api::Error> for Error {
	fn from(value: nvim::api::Error) -> Self {
		Self::from(nvim::Error::Api(value))
	}
}

impl From<nvim::libuv::Error> for Error {
	fn from(value: nvim::libuv::Error) -> Self {
		Self::from(nvim::Error::Libuv(value))
	}
}

impl LogErr for Error {
	fn log_err(&self) {
		nvim::print!("Error: {self}");
	}

	fn log_error(self) -> Self {
		self.log_err();
		self
	}
}
