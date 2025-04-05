use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskManagerConfig {
	pub progress_interval: Duration,
}
