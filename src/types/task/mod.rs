mod list;

pub use list::*;

/// A file system task.
pub enum Task {
	List(List),
}
