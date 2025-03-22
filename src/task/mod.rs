use crate::error::TaskError;

mod copy;
mod create;
mod dfs;
mod list;
mod rename;

pub(crate) use copy::Copy;
pub(crate) use create::Create;
pub(crate) use list::List;
pub(crate) use rename::Rename;

pub(crate) type TaskResult<T> = Result<T, TaskError>;
