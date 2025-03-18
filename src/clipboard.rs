use std::collections::BTreeSet;

use crate::types::File;

pub(crate) struct Clipboard {
	pub copy: BTreeSet<File>,
	pub cut: BTreeSet<File>,
}

impl Clipboard {
	pub(crate) fn new() -> Self {
		Self { copy: BTreeSet::new(), cut: BTreeSet::new() }
	}
}
