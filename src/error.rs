use std::{fmt, io};

use nvim_oxi::{self as nvim, api::Buffer};

use crate::{
	traits::LogErr,
	types::{List, Rename},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Io({0})")]
	Io(#[from] io::Error),

	#[error("Nvim({0})")]
	Nvim(#[from] nvim::Error),

	#[error("SendList({0:?})")]
	SendList(crossbeam_channel::SendError<List>),

	#[error("SendRename({0:?})")]
	SendRename(crossbeam_channel::SendError<Rename>),

	#[error("RecvError({0})")]
	RecvResult(#[from] std::sync::mpsc::RecvError),

	#[error("NoExplorer({0})")]
	NoExplorer(Buffer),

	#[error("NoFile({0})")]
	NoFile(usize),

	#[error("UnknownFile")]
	UnknownFile,

	#[error("Fmt({0})")]
	Fmt(#[from] fmt::Error),

	#[error("TaskError({0})")]
	Task(#[from] TaskError),

	#[error("BufferMismatch")]
	BufferMismatch,

	#[error("NoInputFn")]
	NoInputFn,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
	#[error("Io({0})")]
	Io(#[from] io::Error),

	#[error("Cancelled")]
	Cancelled,
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

impl From<nvim::lua::Error> for Error {
	fn from(value: nvim::lua::Error) -> Self {
		Self::from(nvim::Error::Lua(value))
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
