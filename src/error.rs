use std::{fmt, io};

use nvim_oxi::{self as nvim, api::Buffer};

use crate::{types::List, LogErr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Io({0})")]
	Io(#[from] io::Error),

	#[error("Nvim({0})")]
	Nvim(#[from] nvim::Error),

	#[error("SendList({0:?})")]
	SendList(crossbeam_channel::SendError<List>),

	#[error("RecvError({0})")]
	RecvResult(#[from] std::sync::mpsc::RecvError),

	#[error("NoExplorer({0})")]
	NoExplorer(Buffer),

	#[error("NoFile({0})")]
	NoFile(usize),

	#[error("Cancelled")]
	Cancelled,

	#[error("Fmt({0})")]
	Fmt(#[from] fmt::Error),
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
