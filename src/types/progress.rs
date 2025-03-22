use std::sync::atomic::AtomicU64;

#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Progress {
	pub total: AtomicU64,
	pub done: AtomicU64,
}
