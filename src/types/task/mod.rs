mod list;

pub use list::*;

/// A file system task.
#[derive(Debug)]
pub enum Task {
	List(List),
}
