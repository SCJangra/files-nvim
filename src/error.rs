use std::{fmt, io, path::StripPrefixError};

use crossbeam_channel::RecvError;
use nvim_oxi::{self as nvim, api::Buffer};

use crate::traits::LogErr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Io({0})")]
	Io(#[from] io::Error),

	#[error("Nvim({0})")]
	Nvim(#[from] nvim::Error),

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

	#[error("RecvMsg({0})")]
	RecvMsg(#[from] crossbeam_channel::TryRecvError),

	#[error("InvalidMode")]
	InvalidMode,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
	#[error("Io({0})")]
	Io(#[from] io::Error),

	#[error("Cancelled")]
	Cancelled,

	#[error("StripPrefix({0})")]
	StripPrefix(#[from] StripPrefixError),

	#[error("NotUtf8Path")]
	NotUtf8Path,

	#[error("NotUtf8FileNme")]
	NotUtf8FileNme,

	#[error("Recv({0})")]
	Recv(#[from] RecvError),
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
