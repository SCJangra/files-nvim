use crate::error::TaskError;

mod list;
mod rename;
mod copy;

pub(crate) use list::List;
pub(crate) use rename::Rename;

pub(crate) type TaskResult<T> = Result<T, TaskError>;
