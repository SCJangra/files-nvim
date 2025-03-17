use crate::types::File;

pub(crate) struct Clipboard {
	pub files: Vec<File>,
	pub action: CbAction,
}

#[derive(Clone, Copy)]
pub(crate) enum CbAction {
	Copy,
	Cut,
}

impl Clipboard {
	pub(crate) fn new() -> Self {
		Self { files: Vec::new(), action: CbAction::Copy }
	}
}
