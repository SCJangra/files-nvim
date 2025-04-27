use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskManagerConfig {
	pub progress: TaskProgressConfig,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskProgressConfig {
	pub interval: Duration,
	pub fill_char: String,
}
