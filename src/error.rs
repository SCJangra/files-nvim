use nvim_oxi::{self as oxi, api::Buffer};

use crate::{types::Task, LogErr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Io({0})")]
	Io(#[from] tokio::io::Error),

	#[error("Oxi({0})")]
	Oxi(#[from] oxi::Error),

	#[error("SendTask({0:?})")]
	SendTask(#[from] tokio::sync::mpsc::error::SendError<Task>),

	#[error("RecvError({0})")]
	RecvResult(#[from] std::sync::mpsc::RecvError),

	#[error("NoExplorer({0})")]
	NoExplorer(Buffer),

	#[error("NoFile({0})")]
	NoFile(usize),
}

impl From<oxi::api::Error> for Error {
	fn from(value: oxi::api::Error) -> Self {
		Self::from(oxi::Error::Api(value))
	}
}

impl From<oxi::libuv::Error> for Error {
	fn from(value: oxi::libuv::Error) -> Self {
		Self::from(oxi::Error::Libuv(value))
	}
}

impl LogErr for Error {
	fn log_err(&self) {
		nvim_oxi::print!("Error: {self}");
	}

	fn log_error(self) -> Self {
		self.log_err();
		self
	}
}
