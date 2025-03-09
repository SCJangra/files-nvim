use crate::types::File;

pub(crate) struct List {
	pub files: Vec<File>,
}

impl List {
	pub fn new(files: Vec<File>) -> Self {
		Self { files }
	}
}
