use std::sync::atomic::AtomicU64;

use crate::traits::WriteExt;

#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Progress {
	pub total: AtomicU64,
	pub done: AtomicU64,
}

impl Progress {
	pub fn to_size_string(&self, sep: &str) -> String {
		let mut s = String::new();
		s.write_prog_size(self, sep).unwrap();
		s
	}

	#[allow(unused)]
	pub fn to_count_string(&self, sep: &str) -> String {
		let mut s = String::new();
		s.write_prog_count(self, sep).unwrap();
		s
	}
}
