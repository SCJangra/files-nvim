pub fn bytes_to_size(bytes: u64) -> (f64, &'static str) {
	let bytes: f64 = bytes as f64;

	const KB: f64 = 1000.0;
	const MB: f64 = KB * 1000.0;
	const GB: f64 = MB * 1000.0;
	const TB: f64 = GB * 1000.0;
	const PB: f64 = TB * 1000.0;

	if (0.0..KB).contains(&bytes) {
		(bytes, "B")
	} else if (KB..MB).contains(&bytes) {
		(bytes / KB, "K")
	} else if (MB..GB).contains(&bytes) {
		(bytes / MB, "M")
	} else if (GB..TB).contains(&bytes) {
		(bytes / GB, "G")
	} else if (TB..PB).contains(&bytes) {
		(bytes / TB, "T")
	} else if (PB..).contains(&bytes) {
		(bytes / PB, "P")
	} else {
		unreachable!()
	}
}

pub fn trim_str(val: &str, width: usize) -> (&str, &'static str) {
	match val.len() > width {
		true => (&val[..width.saturating_sub(2)], ".."),
		false => (val, ""),
	}
}
