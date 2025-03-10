use crate::{error::TaskError, types::File};

pub(crate) enum Msg {
	List(Vec<File>),
	TaskDone(usize),
	Rename(usize, String),
}

pub(crate) type MsgResult = Result<Msg, TaskError>;
