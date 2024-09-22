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

impl<T> LogErr for Result<T, Error> {
	fn log_err(self) -> Self {
		if let Err(ref err) = self {
			nvim_oxi::print!("Error: {err}");
		}

		self
	}
}
