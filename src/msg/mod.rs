mod list;

pub(crate) use list::List;

use crate::error::TaskError;

pub(crate) enum Msg {
	List(List),
	TaskDone(usize),
	Rename(usize, String),
}

pub(crate) type MsgResult = Result<Msg, TaskError>;
