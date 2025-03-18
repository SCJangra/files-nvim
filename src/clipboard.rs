use std::collections::BTreeSet;

use crate::types::File;

pub(crate) struct Clipboard {
	pub files: BTreeSet<File>,
	pub action: CbAction,
}

#[derive(Clone, Copy)]
pub(crate) enum CbAction {
	Copy,
	Cut,
}

impl Clipboard {
	pub(crate) fn new() -> Self {
		Self { files: BTreeSet::new(), action: CbAction::Copy }
	}
}
